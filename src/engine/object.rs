use super::{prelude::*, permanent};

pub struct Object {
    pub card: Option<CardID>,
    pub resolve: ObjectResolve,
} 

pub enum ObjectResolve {
    CreatePerm(Permanent),
    AbilityActivate(AbilityID) 
}

impl From<Permanent> for Object {
    fn from(perm: Permanent) -> Self {
        Object {
            card: perm.card,
            resolve: ObjectResolve::CreatePerm(perm)
        }
    }
}
