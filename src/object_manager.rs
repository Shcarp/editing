use crate::{animation::{Animatable, AnimatableExt}, element::Renderable, render_control::{UpdateBody, UpdateMessage, UpdateType}};
use glam::DVec2;
use std::{
    any::Any, cell::RefCell, collections::{HashMap, VecDeque}, rc::Rc
};

#[derive(Debug)]
struct ObjectData {
    object: Rc<RefCell<Box<dyn Renderable >>>,
    last_update: f64,
    position: DVec2,
}

#[derive(Debug)]
struct AnimationObjectData {
    object: Rc<RefCell<Box<dyn Animatable >>>,
    last_update: f64,
}

#[derive(Debug)]
pub struct ObjectManager {
    objects: HashMap<String, ObjectData>,
    update_queue: VecDeque<String>,
    total_time: f64,

    animatable_objects: HashMap<String, AnimationObjectData>,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            update_queue: VecDeque::new(),
            total_time: 0.0,
            animatable_objects: HashMap::new(),
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

        self.objects.insert(id.clone(), object_data);
        self.update_queue.push_back(id);
    }

    pub fn add_animatable(&mut self, object: Rc<RefCell<Box<dyn Animatable>>>) {
        let id = object.borrow().id().value().to_string();
        let object_data = AnimationObjectData {
            object: object.clone(),
            last_update: self.total_time,
        };
        self.animatable_objects.insert(id, object_data);
    }

    pub fn get_animatable_objects(&self) -> HashMap<String, Rc<RefCell<Box<dyn Animatable>>>> {
        let mut objects = HashMap::new();
        for object in self.animatable_objects.iter() {
            objects.insert(object.0.clone(), object.1.object.clone());
        }
        objects
    }

    pub fn transfer_to_objects(&mut self, ids: &Vec<String>) {
        for id in ids {
            if let Some(anim_data) = self.animatable_objects.remove(id) {
                let object = anim_data.object.clone();
                let renderable: Rc<RefCell<Box<dyn Renderable>>> = unsafe {
                    std::mem::transmute(object)
                };
                let position = DVec2::from(renderable.borrow().position());
                let object_data = ObjectData {
                    object: renderable,     
                    last_update: self.total_time,
                    position,
                };
                self.objects.insert(id.clone(), object_data);
                self.update_queue.push_back(id.clone());
            }
        }
    }

    pub fn transfer_to_animatable(&mut self, ids: &[String]) {
        for id in ids {
            if let Some(object_data) = self.objects.remove(id) {
                let object: Rc<RefCell<Box<dyn Renderable>>> = object_data.object;

                let object_check= object.clone();
                let animation_object: Rc<RefCell<Box<dyn Any>>> = unsafe {
                    std::mem::transmute(object_check)
                };

                if let Ok(animatable) = animation_object.try_into_animatable()
                {
                    self.animatable_objects.insert(id.clone(), AnimationObjectData {
                        object: animatable,
                        last_update: self.total_time,
                    });
                } else {
                    self.objects.insert(id.clone(), ObjectData { object, last_update: self.total_time, position: DVec2::new(0.0, 0.0) });
                }
            }
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        if let Some(object_data) = self.objects.remove(id) {
            self.update_queue.retain(|queue_id| queue_id != id);
            Some(object_data.object)
        } else if let Some(anim_data) = self.animatable_objects.remove(id) {
            let object = anim_data.object.clone();
            Some(object.into_renderable())
        } else {
            None
        }
    }

    pub fn get(&self, id: &str) -> Option<Rc<RefCell<Box<dyn Renderable>>>> {
        if let Some(object_data) = self.objects.get(id) {
            Some(object_data.object.clone())
        } else if let Some(anim_data) = self.animatable_objects.get(id) {
            let object = anim_data.object.clone();
            Some(object.into_renderable())
        } else {
            None
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.objects.contains_key(id) || self.animatable_objects.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.objects.len() + self.animatable_objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.animatable_objects.is_empty()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.update_queue.clear();
        self.animatable_objects.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Rc<RefCell<Box<dyn Renderable>>>)> {
        self.objects.iter().map(|(id, data)| (id, &data.object))
    }

    pub fn get_objects(&self) -> Vec<Rc<RefCell<Box<dyn Renderable>>>> {
        let mut objects = Vec::new();
        for (_, data) in self.objects.iter() {
            objects.push(data.object.clone());
        }
        for (_, data) in self.animatable_objects.iter() {
            objects.push(data.object.clone().into_renderable());
        }
        objects
    }

    pub fn update_object_from_message(&mut self, messages: &Vec<UpdateMessage>) {
        let mut update_objects: HashMap<String, Vec<UpdateBody>> = HashMap::new();
        for message in messages.iter() {
            if let UpdateMessage::Update(update_body) = message {
                match &update_body.update_type {
                    UpdateType::ObjectUpdate(id) => {
                        update_objects
                            .entry(id.clone())
                            .or_insert_with(Vec::new)
                            .push(update_body.clone());
                    }
                    _ => {}
                }
            }
        }

        for (object_id, updates) in update_objects.iter() {
            match self.objects.get_mut(object_id) {
                Some(data) => {
                    let mut object = data.object.borrow_mut();
                    for update in updates.iter() {
                        match &update.update_type {
                            UpdateType::ObjectUpdate(id) => {
                                if id == object_id {
                                    object.update(update.data.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                None => todo!(),
            }
        }
    }
}


// 为 Rc<RefCell<dyn Any>> 定义一个 trait 来安全地尝试转换
trait TryIntoAnimatable {
    fn try_into_animatable(self) -> Result<Rc<RefCell<Box<dyn Animatable>>>, Self> where Self: Sized;
}

impl TryIntoAnimatable for Rc<RefCell<Box<dyn Any>>> {
    fn try_into_animatable(self) -> Result<Rc<RefCell<Box<dyn Animatable>>>, Self> {
        if let Ok(animatable) = self.try_borrow() {
            if animatable.is::<Box<dyn Animatable>>() {
                let result = Ok(unsafe { std::mem::transmute(self.clone()) });
                drop(animatable);
                result
            } else {
                drop(animatable);
                Err(self.clone())
            }
        } else {
            Err(self.clone())
        }
    }
}
