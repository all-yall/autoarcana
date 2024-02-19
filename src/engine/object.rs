use super::{prelude::*, permanent};

pub struct Object {
    pub card: Option<CardID>,
    pub resolve: ObjectResolve,
} 

pub enum ObjectResolve {
    CreateLand(Permanent),
    CreatePerm(Permanent),
    AbilityActivate(AbilityID) 
}

impl From<Permanent> for Object {
    fn from(perm: Permanent) -> Self {
        let card = perm.card;

        let resolve = if perm.type_line.is(CardType::Land) {
            ObjectResolve::CreateLand(perm)
        } else {
            ObjectResolve::CreatePerm(perm)
        };

        Object {
            card,
            resolve,
        }
    }
}
