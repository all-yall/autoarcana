use super::abilities::*;
use super::card_plays::*;
use crate::engine::prelude::*;

pub fn get_card(name: &str) -> LatentCard {
    use CardType::*;
    use CardSuperType::*;
    use ManaType::*;
    let mut ret = match name {
        "mountain" => LatentCard::new(
            "Mountain".into(),
            ManaCost::empty(),
            "One day, night will come to these mountains.".into(), 
            TypeLine::empty().add(Basic).add(Land).add("Mountain"),
            vec![ 
                LatentAbility {
                    class: AbilityClass::Activated(AbilityCost::empty().with_tap(), AddManaEffect::new(Red)),
                    description: "Add one red mana".into(),
                },
            ],
            vec![],
            None
        ),

        "miraris wake" => LatentCard::new(
            "Mirari's Wake".into(),
            ManaCost::empty(),
            "Even after a false god tore the magic from Dominaria, power still radiated from the Mirari sword that slew her.".into(),
            TypeLine::empty().add(Enchantment),
            vec![
                LatentAbility {
                    // TODO implement this static ability
                    class: AbilityClass::Static(NullEffect::new()),
                    description: "Creatures you control get +1/+1.".into(),
                },
                LatentAbility {
                    class: AbilityClass::Triggered(MiraisMana::new()),
                    description: "Whenever you tap a land for mana, add one mana of any type that land produced.".into(),
                }
            ],
            vec![],
            None
        ),

        "goblin assailant" => LatentCard::new(
            "Goblin Assailant".into(),
            ManaCost::empty(),
            "What he lacks in patience, intelligence, empathy, lucidity, hygiene, ability to follow orders, self-regard, and discernible skills, he makes up for in sheer chaotic violence.".into(),
            TypeLine::empty().add(Creature).add("Goblin").add("Warrior"),
            vec![],
            vec![],
            Some((2, 2))
        ),

        other => panic!("no card named '{}'", other),
    };

    def_card_plays(&mut ret);
    ret
}
