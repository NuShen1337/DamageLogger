// use function_name::named;

// use crate::{
//     battle::BattleContext,
//     kreide::{
//         helpers::{get_avatar_from_entity, get_monster_from_entity},
//         types::{
//             RPG_GameCore_EntityType, RPG_GameCore_GameEntity,
//             RPG_GameCore_TurnBasedAbilityComponent,
//         },
//     },
//     models::{
//         events::{Event, OnEntityDefeatedEvent},
//         misc::Avatar,
//     },
//     safe_call,
//     subscribers::battle::ON_KILL_ENEMY_Detour,
// };

// #[named]
// pub fn on_entity_defeated(instance: usize, entity: usize) {
//     log::debug!(function_name!());
//     ON_KILL_ENEMY_Detour.call(instance, entity);
//     safe_call!(unsafe {
//         let ability_component = RPG_GameCore_TurnBasedAbilityComponent(instance);
//         let owner_ref = ability_component._OwnerRef().unwrap();

//         if !ability_component
//             .try_check_limbo_wait_heal(owner_ref)
//             .unwrap_or_default()
//         {
//             let entity = RPG_GameCore_GameEntity(entity);
//             if entity._EntityType().unwrap() == RPG_GameCore_EntityType::Avatar {
//                 let e = match get_avatar_from_entity(entity) {
//                     Ok(avatar) => Ok(Event::OnEntityDefeated(OnEntityDefeatedEvent {
//                         killer: avatar,
//                         entity_defeated: get_monster_from_entity(owner_ref).unwrap_or(Avatar {
//                             id: entity._RuntimeID_k__BackingField().unwrap(),
//                             name: String::from("Monster"),
//                         }),
//                     })),
//                     Err(e) => Err(anyhow::anyhow!(
//                         "{} Avatar Event Error: {}",
//                         function_name!(),
//                         e
//                     )),
//                 };
//                 BattleContext::handle_event(e);
//             };
//         }
//     });
// }
