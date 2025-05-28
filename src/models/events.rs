use serde_json::Value;

use super::misc::{Avatar, Enemy, Entity, Skill, Stat, Team};

#[allow(clippy::enum_variant_names)]
pub enum Event {
    OnBattleBegin(OnBattleBeginEvent),
    OnSetBattleLineup(OnSetLineupEvent),
    OnDamage(OnDamageEvent),
    OnTurnBegin(OnTurnBeginEvent),
    OnTurnEnd(OnTurnEndEvent),
    OnUseSkill(OnUseSkillEvent),
    OnBattleEnd,
    OnUpdateWave(OnUpdateWaveEvent),
    OnUpdateCycle(OnUpdateCycleEvent),
    OnStatChange(OnStatChangeEvent),
    // OnEntityDefeated(OnEntityDefeatedEvent),
    OnUpdateTeamFormation(OnUpdateTeamFormationEvent),
    OnInitializeEnemy(OnInitializeEnemyEvent),
}

pub struct OnBattleBeginEvent {
    pub max_waves: u32,
    pub max_cycles: u32,
    pub stage_id: u32,
}

pub struct OnUpdateWaveEvent {
    pub wave: u32,
}

pub struct OnUpdateCycleEvent {
    pub cycle: u32,
}

pub struct OnTurnBeginEvent {
    pub action_value: f64,
    pub turn_owner: Option<Avatar>,
    pub monster_hps: Vec<Value>,
    pub extra_data: Value,
}

pub struct OnTurnEndEvent {
    pub monster_hps: Vec<Value>,
}

pub struct OnUseSkillEvent {
    pub avatar: Avatar,
    pub skill: Skill,
}

pub struct OnSetLineupEvent {
    pub avatars: Vec<Avatar>,
}

pub struct OnDamageEvent {
    pub attacker: Avatar,
    pub damage: f64,
    pub damage_type: &'static str,
}

// pub struct OnEntityDefeatedEvent {
//     pub killer: Avatar,
//     pub entity_defeated: Avatar,
// }

pub struct OnStatChangeEvent {
    pub entity: Entity,
    pub stat: Stat,
}

pub struct OnUpdateTeamFormationEvent {
    pub entities: Vec<Entity>,
    pub team: Team,
}

pub struct OnInitializeEnemyEvent {
    pub enemy: Enemy,
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}
