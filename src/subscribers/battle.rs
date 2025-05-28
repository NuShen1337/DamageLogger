#![allow(clippy::too_many_arguments)]
use crate::kreide::{
    helpers::fixpoint_to_raw,
    types::{
        FixPoint, RPG_Client_BattleAssetPreload, RPG_GameCore_AbilityProperty,
        RPG_GameCore_BattleInstance, RPG_GameCore_MonsterDataComponent,
        RPG_GameCore_SkillCharacterComponent, RPG_GameCore_TeamFormationComponent,
        RPG_GameCore_TurnBasedAbilityComponent, RPG_GameCore_TurnBasedGameMode, DMFMLMJKKHB,
        MMNDIEBMDNL,
    },
};

use super::*;

use anyhow::Result;
use function_name::named;
use retour::static_detour;

#[macro_export]
macro_rules! safe_call {
    ($body:expr) => {
        #[allow(unused_must_use)]
        match microseh::try_seh(|| std::panic::catch_unwind(|| $body)) {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(panic)) => {
                let backtrace = std::backtrace::Backtrace::capture();
                let msg = panic
                    .downcast_ref::<&str>()
                    .map(|s| format!("Panic message: {}", s))
                    .or_else(|| {
                        panic
                            .downcast_ref::<String>()
                            .map(|s| format!("Panic message: {}", s))
                    })
                    .unwrap_or_else(|| format!("Unknown panic: {:#?}", panic));

                let message = format!(
                    "{} panicked {}\nBacktrace:\n{}",
                    function_name!(),
                    msg,
                    backtrace
                );

                log::error!("{}", message);
                Err(anyhow::anyhow!(message).context("Panic occurred"))
            }
            Err(seh) => {
                let backtrace = std::backtrace::Backtrace::capture();
                let message = format!(
                    "{} triggered SEH exception: {:?}\nBacktrace:\n{}",
                    function_name!(),
                    seh,
                    backtrace
                );

                log::error!("{}", message);
                Err(anyhow::anyhow!(message).context("SEH occurred"))
            }
        }
    };
}

static_detour! {
    pub(super) static ON_DAMAGE_Detour: fn(
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        bool,
        usize
    ) -> bool;
    pub(super) static ON_COMBO_Detour: fn(usize);
    pub(super) static ON_USE_SKILL_Detour: fn(usize,i32, usize, bool, i32);
    pub(super) static ON_SET_LINEUP_Detour: fn(usize, bool, usize);
    pub(super) static ON_BATTLE_BEGIN_Detour: fn(usize);
    pub(super) static ON_BATTLE_END_Detour: fn(usize);
    pub(super) static ON_TURN_BEGIN_Detour: fn(usize);
    pub(super) static ON_TURN_END_Detour: fn(usize, i32) -> usize;
    pub(super) static ON_UPDATE_WAVE_Detour: fn (usize);
    pub(super) static ON_UPDATE_CYCLE_Detour: fn (usize) -> u32;

    pub(super) static ON_DIRECT_CHANGE_HP_Detour: fn (usize, i32, FixPoint, usize);
    pub(super) static ON_DIRECT_DAMAGE_HP_Detour: fn (usize, FixPoint, i32, usize, FixPoint, usize);
    pub(super) static ON_STAT_CHANGE_Detour: fn (usize, RPG_GameCore_AbilityProperty, i32, FixPoint, usize);
    pub(super) static ON_KILL_ENEMY_Detour: fn(usize, usize);
    pub(super) static ON_UPDATE_TEAM_FORMATION_Detour: fn(usize);
    pub(super) static ON_INITIALIZE_ENEMY_Detour: fn(usize, usize);

}

pub(super) static mut TURN_BASED_GAME_MODE_REF: Option<RPG_GameCore_TurnBasedGameMode> = None;

#[named]
pub(super) fn get_elapsed_av(game_mode: usize) -> f64 {
    log::debug!(function_name!());
    let elapsed_action_delay =
        RPG_GameCore_TurnBasedGameMode(game_mode)._ElapsedActionDelay_k__BackingField();
    if let Ok(elased_action_delay) = elapsed_action_delay {
        fixpoint_to_raw(&elased_action_delay) * 10f64
    } else {
        log::warn!("ElapsedActionDelay was null! returning 0.0");
        0.0
    }
}

#[named]
pub(super) fn get_battle_instance() -> Result<RPG_GameCore_BattleInstance> {
    unsafe {
        match TURN_BASED_GAME_MODE_REF.map(|i| i._OwnerBattleInstanceRef_k__BackingField())
        {
            Some(Ok(i)) => Ok(i),
            Some(Err(err)) => Err(anyhow::anyhow!(
                "{} failed to get battle instance {err}",
                function_name!()
            )),
            None => {
                Err(anyhow::anyhow!(
                    "get_battle_instance() is called, but there was no reference to RPG.GameCore.TurnBasedGameMode"
                ))
            }
        }
    }
}

pub fn subscribe() -> Result<()> {
    unsafe {
        subscribe_function!(
            ON_DAMAGE_Detour,
            DMFMLMJKKHB::get_class()
                .unwrap()
                .find_method_by_name("OMPLOLLELLK")
                .unwrap()
                .va(),
            on_damage::on_damage
        );
        subscribe_function!(
            ON_COMBO_Detour,
            MMNDIEBMDNL::get_class()
                .unwrap()
                .find_method_by_name("FECMPGBOBOI")
                .unwrap()
                .va(),
            on_combo::on_combo
        );
        subscribe_function!(
            ON_USE_SKILL_Detour,
            RPG_GameCore_SkillCharacterComponent::get_class()
                .unwrap()
                .find_method_by_name("UseSkill")
                .unwrap()
                .va(),
            on_use_skill::on_use_skill
        );
        subscribe_function!(
            ON_SET_LINEUP_Detour,
            RPG_Client_BattleAssetPreload::get_class()
                .unwrap()
                .find_method_by_name("InBattleAssetPreload")
                .unwrap()
                .va(),
            on_set_lineup::on_set_lineup
        );
        subscribe_function!(
            ON_BATTLE_BEGIN_Detour,
            RPG_GameCore_TurnBasedGameMode::get_class()
                .unwrap()
                .find_method_by_name("_GameModeBegin")
                .unwrap()
                .va(),
            on_battle_begin::on_battle_begin
        );
        subscribe_function!(
            ON_BATTLE_END_Detour,
            RPG_GameCore_TurnBasedGameMode::get_class()
                .unwrap()
                .find_method_by_name("_GameModeEnd")
                .unwrap()
                .va(),
            on_battle_end::on_battle_end
        );
        subscribe_function!(
            ON_TURN_BEGIN_Detour,
            RPG_GameCore_TurnBasedGameMode::get_class()
                .unwrap()
                .find_method_by_name("DoTurnPrepareStartWork")
                .unwrap()
                .va(),
            on_turn_begin::on_turn_begin
        );
        subscribe_function!(
            ON_TURN_END_Detour,
            RPG_GameCore_TurnBasedAbilityComponent::get_class()
                .unwrap()
                .find_method_by_name("ProcessOnLevelTurnActionEndEvent")
                .unwrap()
                .va(),
            on_turn_end::on_turn_end
        );
        subscribe_function!(
            ON_UPDATE_WAVE_Detour,
            RPG_GameCore_TurnBasedGameMode::get_class()
                .unwrap()
                .find_method_by_name("UpdateCurrentWaveCount")
                .unwrap()
                .va(),
            on_update_wave::on_update_wave
        );
        subscribe_function!(
            ON_UPDATE_CYCLE_Detour,
            RPG_GameCore_TurnBasedGameMode::get_class()
                .unwrap()
                .find_method_by_name("get_ChallengeTurnLeft")
                .unwrap()
                .va(),
            on_update_cycle::on_update_cycle
        );
        subscribe_function!(
            ON_DIRECT_CHANGE_HP_Detour,
            RPG_GameCore_TurnBasedAbilityComponent::get_class()
                .unwrap()
                .find_method_by_name("DirectChangeHP")
                .unwrap()
                .va(),
            on_direct_change_hp::on_direct_change_hp
        );
        subscribe_function!(
            ON_DIRECT_DAMAGE_HP_Detour,
            RPG_GameCore_TurnBasedAbilityComponent::get_class()
                .unwrap()
                .find_method_by_name("DirectDamageHP")
                .unwrap()
                .va(),
            on_direct_damage_hp::on_direct_damage_hp
        );
        subscribe_function!(
            ON_STAT_CHANGE_Detour,
            RPG_GameCore_TurnBasedAbilityComponent::get_class()
                .unwrap()
                .find_method_by_name("ModifyProperty")
                .unwrap()
                .va(),
            on_stat_change::on_stat_change
        );
        // subscribe_function!(
        //     ON_KILL_ENEMY_Detour,
        //     RPG_GameCore_TurnBasedAbilityComponent::get_class()
        //         .unwrap()
        //         .find_method_by_name("set_KillerEntity")
        //         .unwrap()
        //         .va(),
        //     on_entity_defeated::on_entity_defeated
        // );
        subscribe_function!(
            ON_UPDATE_TEAM_FORMATION_Detour,
            RPG_GameCore_TeamFormationComponent::get_class()
                .unwrap()
                .find_method_by_name("_RefreshTeammateIndex")
                .unwrap()
                .va(),
            on_update_team_formation::on_update_team_formation
        );
        subscribe_function!(
            ON_INITIALIZE_ENEMY_Detour,
            RPG_GameCore_MonsterDataComponent::get_class()
                .unwrap()
                .find_method_by_name("OnAbilityCharacterInitialized")
                .unwrap()
                .va(),
            on_initialize_enemy::on_initialize_enemy
        );
        Ok(())
    }
}

// Interesting hierarchy
// GameEntity -> _OwnerWorldRef -> BattleInstanceRef

// Is this good for hovering HP bar?
// public UnityEngine.Transform TryGetSelectPointFromHitBoxGroup(RPG.GameCore.GameEntity) { }
