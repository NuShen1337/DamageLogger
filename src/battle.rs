use std::sync::{LazyLock, Mutex, MutexGuard};

use anyhow::{Context, Result};

use crate::{
    models::{
        events::{
            Event, OnBattleBeginEvent, OnDamageEvent, OnInitializeEnemyEvent, OnSetLineupEvent,
            OnStatChangeEvent, OnTurnBeginEvent, OnTurnEndEvent, OnUpdateCycleEvent,
            OnUpdateTeamFormationEvent, OnUpdateWaveEvent, OnUseSkillEvent,
        },
        misc::{Avatar, BattleEntity, BattleStats, Enemy, Entity, Team, TurnInfo},
        packets::Packet,
    },
    server,
};

#[derive(Default, Clone, Copy)]
pub enum BattleState {
    #[default]
    Preparing,
    Started,
    Ended,
}

// Data that aren't meant to be exposed in the API
// And is only for the overlay frontend
// pub struct BattleContextInternal {
//     pub relative_action_value: f64,
// }

#[derive(Default, Clone)]
pub struct BattleContext {
    pub state: BattleState,
    pub avatar_lineup: Vec<Avatar>,
    pub battle_avatars: Vec<BattleEntity>,
    pub enemies: Vec<Enemy>,
    pub enemy_lineup: Vec<Entity>,
    pub battle_enemies: Vec<BattleEntity>,
    pub turn_history: Vec<TurnInfo>,
    pub av_history: Vec<TurnInfo>,
    pub last_wave_action_value: f64,
    pub action_value: f64,
    pub current_turn_info: TurnInfo,
    pub turn_count: usize,
    pub total_damage: f64,
    // Index w/ lineup index
    // Used to update UI damage when dmg occurs
    pub real_time_damages: Vec<f64>,
    pub max_waves: u32,
    pub wave: u32,
    pub cycle: u32,
    pub stage_id: u32,
    // Not meant to be exposed in the API
    // pub internal: BattleContextInternal,
}

// enum BattleMode {
//     MOC,
//     PF,
//     AS,
//     Other,
// }

static BATTLE_CONTEXT: LazyLock<Mutex<BattleContext>> =
    LazyLock::new(|| Mutex::new(BattleContext::default()));

impl BattleContext {
    pub fn get_instance() -> MutexGuard<'static, Self> {
        BATTLE_CONTEXT.lock().unwrap()
    }

    fn find_lineup_index_by_avatar_id(
        battle_context: &MutexGuard<'static, Self>,
        avatar_id: u32,
    ) -> Option<usize> {
        let res = battle_context
            .avatar_lineup
            .iter()
            .enumerate()
            .find(|(_index, avatar)| avatar.id == avatar_id);
        res.map(|(index, _)| index)
    }

    fn initialize_battle_context(battle_context: &mut MutexGuard<'static, Self>) {
        battle_context.current_turn_info = TurnInfo::default();
        battle_context.turn_history = Vec::new();
        battle_context.av_history = Vec::new();

        battle_context.enemies = Vec::new();
        battle_context.battle_enemies = Vec::new();

        battle_context.turn_count = 0;
        battle_context.total_damage = 0.;
        battle_context.last_wave_action_value = 0.;
        battle_context.action_value = 0.;
        battle_context.max_waves = 0;
        battle_context.wave = 0;
        battle_context.cycle = 0;
        battle_context.stage_id = 0;
    }

    // A word of caution:
    // The lineup is setup first
    fn handle_on_battle_begin_event(
        e: OnBattleBeginEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        battle_context.state = BattleState::Started;
        log::info!("Battle has started");
        log::info!("Max Waves: {}", e.max_waves);
        battle_context.max_waves = e.max_waves;

        // let
        // e.stage_id.to_string().char_indices().nth_back(2).unwrap().0

        Ok(Packet::OnBattleBegin {
            max_waves: e.max_waves,
            max_cycles: e.max_cycles,
            stage_id: e.stage_id,
        })
    }

    fn handle_on_set_lineup_event(
        e: OnSetLineupEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        Self::initialize_battle_context(&mut battle_context);
        battle_context.current_turn_info.avatars_turn_damage = vec![0f64; e.avatars.len()];
        battle_context.real_time_damages = vec![0f64; e.avatars.len()];
        battle_context.avatar_lineup = e.avatars;

        let mut battle_avatars = Vec::new();
        for avatar in &battle_context.avatar_lineup {
            battle_avatars.push(BattleEntity {
                entity: Entity {
                    uid: avatar.id,
                    team: Team::Player,
                },
                battle_stats: BattleStats::default(),
            });
        }
        battle_context.battle_avatars = battle_avatars;

        for avatar in &battle_context.avatar_lineup {
            log::info!("{avatar} was loaded in lineup");
        }

        Ok(Packet::OnSetBattleLineup {
            avatars: battle_context.avatar_lineup.clone(),
        })
    }

    fn handle_on_damage_event(
        e: OnDamageEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        let lineup_index = Self::find_lineup_index_by_avatar_id(&battle_context, e.attacker.id)
            .with_context(|| format!("Could not find avatar {} in lineup", e.attacker.id))?;
        let turn = &mut battle_context.current_turn_info;
        // Record character damage chunk
        turn.avatars_turn_damage[lineup_index] += e.damage;
        battle_context.real_time_damages[lineup_index] += e.damage;
        battle_context.total_damage += e.damage;

        Ok(Packet::OnDamage {
            attacker: e.attacker,
            damage: e.damage,
            damage_type: e.damage_type,
        })
    }

    fn handle_on_turn_begin_event(
        e: OnTurnBeginEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        battle_context.action_value = e.action_value;
        battle_context.current_turn_info.action_value = e.action_value;

        log::info!("AV: {:.2}", e.action_value);

        Ok(Packet::OnTurnBegin {
            action_value: e.action_value,
            turn_owner: e.turn_owner,
            monster_hps: e.monster_hps,
            extra_data: e.extra_data,
        })
    }

    fn handle_on_turn_end_event(
        e: OnTurnEndEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        battle_context.current_turn_info.wave = battle_context.wave;
        battle_context.current_turn_info.cycle = battle_context.cycle;

        let mut turn_info = battle_context.current_turn_info.clone();

        turn_info.monster_hps = e.monster_hps;

        // Calculate net damages
        turn_info.total_damage = if turn_info.avatars_turn_damage.is_empty() {
            0.0
        } else {
            turn_info.avatars_turn_damage.iter().sum()
        };
        battle_context.turn_history.push(turn_info.clone());

        // If same AV, update damage
        if let Some(last_turn) = battle_context.av_history.last_mut() {
            if last_turn.action_value == turn_info.action_value {
                for (i, incoming_dmg) in turn_info.avatars_turn_damage.iter().enumerate() {
                    last_turn.avatars_turn_damage[i] += incoming_dmg;
                }
            } else {
                battle_context.av_history.push(turn_info.clone());
            }
        } else {
            battle_context.av_history.push(turn_info.clone());
        }

        // Logging
        for (i, avatar) in battle_context.avatar_lineup.iter().enumerate() {
            if turn_info.avatars_turn_damage[i] > 0.0 {
                log::info!(
                    "Turn Summary: {} has dealt {:.2} damage",
                    avatar,
                    turn_info.avatars_turn_damage[i]
                );
            }
        }

        if turn_info.total_damage > 0.0 {
            log::info!(
                "Turn Summary: Total damage of {:.2}",
                turn_info.total_damage
            );
        }

        // Restart turn info
        // battle_context.current_turn_info.total_damage = 0.0;
        battle_context.current_turn_info.avatars_turn_damage =
            vec![0f64; battle_context.avatar_lineup.len()];
        battle_context.turn_count += 1;

        Ok(Packet::OnTurnEnd { turn_info })
    }

    // fn handle_on_entity_defeated_event(
    //     e: OnEntityDefeatedEvent,
    //     mut _battle_context: MutexGuard<'static, BattleContext>,
    // ) -> Result<Packet> {
    //     // log::info!("{} has defeated {}", e.attacker);

    //     Ok(Packet::OnEntityDefeated {
    //         killer: e.killer,
    //         entity_defeated: e.entity_defeated,
    //     })
    // }

    fn handle_on_battle_end_event(
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        battle_context.state = BattleState::Ended;

        Ok(Packet::OnBattleEnd {
            avatars: battle_context.avatar_lineup.clone(),
            turn_history: battle_context.turn_history.clone(),
            av_history: battle_context.av_history.clone(),
            turn_count: battle_context.turn_count,
            total_damage: battle_context.total_damage,
            action_value: battle_context.action_value,
            cycle: battle_context.cycle,
            wave: battle_context.wave,
            stage_id: battle_context.stage_id,
        })
    }

    fn handle_on_use_skill_event(
        e: OnUseSkillEvent,
        mut _battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        // log::info!("{} has used {}", e.avatar.uid, e.skill);

        Ok(Packet::OnUseSkill {
            avatar: e.avatar,
            skill: e.skill,
        })
    }

    fn handle_on_update_wave_event(
        e: OnUpdateWaveEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        log::info!("Wave: {}", e.wave);

        battle_context.wave = e.wave;
        Ok(Packet::OnUpdateWave { wave: e.wave })
    }

    fn handle_on_update_cycle_event(
        e: OnUpdateCycleEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        log::info!("Cycle: {}", e.cycle);

        battle_context.cycle = e.cycle;
        Ok(Packet::OnUpdateCycle { cycle: e.cycle })
    }

    fn handle_on_stat_change_event(
        e: OnStatChangeEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        match e.entity.team {
            Team::Player => {
                if let Some(avatar) = battle_context
                    .battle_avatars
                    .iter_mut()
                    .find(|x| x.entity == e.entity)
                {
                    match e.stat {
                        crate::models::misc::Stat::HP(stat) => avatar.battle_stats.hp = stat,
                        crate::models::misc::Stat::Attack(stat) => {
                            avatar.battle_stats.attack = stat
                        }
                        crate::models::misc::Stat::Defense(stat) => {
                            avatar.battle_stats.defense = stat
                        }
                        crate::models::misc::Stat::Speed(stat) => avatar.battle_stats.speed = stat,
                        crate::models::misc::Stat::AV(stat) => avatar.battle_stats.av = stat,
                    }
                }
            }
            Team::Enemy => {
                if let Some(enemy) = battle_context
                    .battle_enemies
                    .iter_mut()
                    .find(|x| x.entity == e.entity)
                {
                    match e.stat {
                        crate::models::misc::Stat::HP(stat) => enemy.battle_stats.hp = stat,
                        crate::models::misc::Stat::Attack(stat) => enemy.battle_stats.attack = stat,
                        crate::models::misc::Stat::Defense(stat) => {
                            enemy.battle_stats.defense = stat
                        }
                        crate::models::misc::Stat::Speed(stat) => enemy.battle_stats.speed = stat,
                        crate::models::misc::Stat::AV(stat) => enemy.battle_stats.av = stat,
                    }
                }
            }
        }

        Ok(Packet::OnStatChange {
            entity: e.entity,
            stat: e.stat,
        })
    }

    fn handle_on_initialize_enemy_event(
        e: OnInitializeEnemyEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        battle_context.enemies.push(e.enemy.clone());
        battle_context.battle_enemies.push(BattleEntity {
            entity: Entity {
                uid: e.enemy.uid,
                team: Team::Enemy,
            },
            battle_stats: BattleStats {
                hp: e.enemy.base_stats.hp,
                ..Default::default()
            },
        });
        Ok(Packet::OnInitializeEnemy { enemy: e.enemy })
    }

    fn handle_on_update_team_formation_event(
        e: OnUpdateTeamFormationEvent,
        mut battle_context: MutexGuard<'static, BattleContext>,
    ) -> Result<Packet> {
        match e.team {
            Team::Player => {}
            Team::Enemy => {
                battle_context.enemy_lineup = e.entities.clone();
            }
        }
        Ok(Packet::OnUpdateTeamFormation {
            entities: e.entities,
            team: e.team,
        })
    }

    pub fn handle_event(event: Result<Event>) {
        let battle_context = Self::get_instance();
        let packet = match event {
            Result::Ok(event) => match event {
                Event::OnBattleBegin(e) => Self::handle_on_battle_begin_event(e, battle_context),
                Event::OnSetBattleLineup(e) => Self::handle_on_set_lineup_event(e, battle_context),
                Event::OnDamage(e) => Self::handle_on_damage_event(e, battle_context),
                Event::OnTurnBegin(e) => Self::handle_on_turn_begin_event(e, battle_context),
                Event::OnTurnEnd(e) => Self::handle_on_turn_end_event(e, battle_context),
                // Event::OnEntityDefeated(e) => {
                //     Self::handle_on_entity_defeated_event(e, battle_context)
                // }
                Event::OnBattleEnd => Self::handle_on_battle_end_event(battle_context),
                Event::OnUseSkill(e) => Self::handle_on_use_skill_event(e, battle_context),
                Event::OnUpdateWave(e) => Self::handle_on_update_wave_event(e, battle_context),
                Event::OnUpdateCycle(e) => {
                    if e.cycle == battle_context.cycle {
                        return;
                    }
                    Self::handle_on_update_cycle_event(e, battle_context)
                }
                Event::OnStatChange(e) => Self::handle_on_stat_change_event(e, battle_context),
                Event::OnInitializeEnemy(e) => {
                    Self::handle_on_initialize_enemy_event(e, battle_context)
                }
                Event::OnUpdateTeamFormation(e) => {
                    Self::handle_on_update_team_formation_event(e, battle_context)
                }
            },
            Err(e) => Ok({
                log::error!("{e}");
                Packet::Error { msg: e.to_string() }
            }),
        };

        match packet {
            Result::Ok(packet) => {
                server::broadcast(packet);
            }
            Err(e) => log::error!("Packet Error: {e}"),
        };
    }
}
