use crate::element::Renderable;
use glam::DVec2;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
}; // Assuming you're using glam for vector math

const GRID_CELL_SIZE: f64 = 100.0;
const UPDATE_PRIORITY_THRESHOLD: f64 = 0.1;

#[derive(Debug)]
struct GridCell {
    objects: Vec<String>,
}

#[derive(Debug)]
struct Grid {
    cells: HashMap<(i32, i32), GridCell>,
}

impl Grid {
    fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    fn insert(&mut self, id: &str, position: DVec2) {
        let cell_pos = Self::world_to_cell(position);
        self.cells
            .entry(cell_pos)
            .or_insert_with(|| GridCell {
                objects: Vec::new(),
            })
            .objects
            .push(id.to_string());
    }

    fn remove(&mut self, id: &str, position: DVec2) {
        let cell_pos = Self::world_to_cell(position);
        if let Some(cell) = self.cells.get_mut(&cell_pos) {
            cell.objects.retain(|obj_id| obj_id != id);
            if cell.objects.is_empty() {
                self.cells.remove(&cell_pos);
            }
        }
    }

    fn update_position(&mut self, id: &str, old_pos: DVec2, new_pos: DVec2) {
        let old_cell = Self::world_to_cell(old_pos);
        let new_cell = Self::world_to_cell(new_pos);
        if old_cell != new_cell {
            self.remove(id, old_pos);
            self.insert(id, new_pos);
        }
    }

    fn get_nearby_objects(&self, position: DVec2, radius: f64) -> Vec<String> {
        let mut nearby_objects = Vec::new();
        let cell_radius = (radius / GRID_CELL_SIZE).ceil() as i32;
        let center = Self::world_to_cell(position);

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                if let Some(cell) = self.cells.get(&(center.0 + dx, center.1 + dy)) {
                    nearby_objects.extend(cell.objects.iter().cloned());
                }
            }
        }
        nearby_objects
    }

    fn world_to_cell(position: DVec2) -> (i32, i32) {
        (
            (position.x / GRID_CELL_SIZE).floor() as i32,
            (position.y / GRID_CELL_SIZE).floor() as i32,
        )
    }
}

#[derive(Debug)]
struct ObjectData {
    object: Rc<RefCell<Box<dyn Renderable>>>,
    last_update: f64,
    position: DVec2,
}

#[derive(Debug)]
pub struct ObjectManager {
    objects: HashMap<String, ObjectData>,
    grid: Grid,
    update_queue: VecDeque<String>,
    total_time: f64,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            grid: Grid::new(),
            update_queue: VecDeque::new(),
            total_time: 0.0,
        }
    }

    pub fn add<T>(&mut self, object: T)
    where
        T: Renderable + 'static,
    {
        let id = object.id().value().to_string();
        let position = DVec2::new(object.position().0, object.position().1);
        let object_data = ObjectData {
            object: Rc::new(RefCell::new(Box::new(object))),
            last_update: self.total_time,
            position,
        };

        self.grid.insert(&id, position);
        self.objects.insert(id.clone(), object_data);
        self.update_queue.push_back(id);
    }

    pub fn remove(&mut self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        if let Some(object_data) = self.objects.remove(id) {
            self.grid.remove(id, object_data.position);
            self.update_queue.retain(|queue_id| queue_id != id);
            Some(object_data.object)
        } else {
            None
        }
    }

    pub fn get(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        self.objects.get(id).map(|data| data.object.clone())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.objects.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.grid = Grid::new();
        self.update_queue.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Rc<RefCell<Box<dyn Renderable>>>)> {
        self.objects.iter().map(|(id, data)| (id, &data.object))
    }

    pub fn update_batch(&mut self, delta_time: f64, max_updates: usize) {
        self.total_time += delta_time;
        let mut updates_performed = 0;

        while updates_performed < max_updates && !self.update_queue.is_empty() {
            if let Some(id) = self.update_queue.pop_front() {
                if let Some(object_data) = self.objects.get_mut(&id) {
                    let old_position = object_data.position;
                    object_data.object.borrow_mut().update(delta_time);
                    let new_position = DVec2::new(
                        object_data.object.borrow().position().0,
                        object_data.object.borrow().position().1,
                    );

                    if old_position != new_position {
                        self.grid.update_position(&id, old_position, new_position);
                        object_data.position = new_position;
                    }

                    object_data.last_update = self.total_time;
                    updates_performed += 1;

                    // Re-queue with lower priority
                    self.update_queue.push_back(id);
                }
            }
        }

        // Prioritize objects that haven't been updated recently
        self.update_queue.make_contiguous().sort_by(|a, b| {
            let time_a = self
                .objects
                .get(a)
                .map(|data| data.last_update)
                .unwrap_or(0.0);
            let time_b = self
                .objects
                .get(b)
                .map(|data| data.last_update)
                .unwrap_or(0.0);
            time_a.partial_cmp(&time_b).unwrap()
        });
    }

    pub fn update_visible(&mut self, camera_position: DVec2, view_radius: f64, delta_time: f64) {
        let visible_ids = self.grid.get_nearby_objects(camera_position, view_radius);
        for id in visible_ids {
            if let Some(object_data) = self.objects.get_mut(&id) {
                if self.total_time - object_data.last_update > UPDATE_PRIORITY_THRESHOLD {
                    let old_position = object_data.position;
                    object_data.object.borrow_mut().update(delta_time);
                    let new_position = DVec2::new(
                        object_data.object.borrow().position().0,
                        object_data.object.borrow().position().1,
                    );

                    if old_position != new_position {
                        self.grid.update_position(&id, old_position, new_position);
                        object_data.position = new_position;
                    }

                    object_data.last_update = self.total_time;
                }
            }
        }
    }

    pub fn update_all(&mut self, delta_time: f64) {
        self.total_time += delta_time;
        for (id, object_data) in self.objects.iter_mut() {
            let old_position = object_data.position;
            object_data.object.borrow_mut().update(delta_time);
            let new_position = DVec2::new(
                object_data.object.borrow().position().0,
                object_data.object.borrow().position().1,
            );

            if old_position != new_position {
                self.grid.update_position(id, old_position, new_position);
                object_data.position = new_position;
            }

            object_data.last_update = self.total_time;
        }
    }

    pub fn get_objects(&self) -> Vec<Rc<RefCell<Box<dyn Renderable>>>> {
        self.objects
            .values()
            .map(|data| data.object.clone())
            .collect()
    }
}
