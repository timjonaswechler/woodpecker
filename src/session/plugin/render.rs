//! Keep the render-world bootstrap behind the first completed counted tick.
//!
//! Startup may create active cameras whose derived render data is still unprepared:
//! application PostUpdate has not run yet. Neither extraction nor rendering may
//! consume that data. Once a tick completes, the original render path resumes,
//! including during subsequent idle control iterations. No main-world camera or
//! application schedule is changed here. The gate travels with RenderApp when
//! Bevy moves it to the pipelined rendering thread.
use bevy::{
    app::App,
    ecs::schedule::ScheduleLabel,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        view::visibility::{
            RenderExtractedShadowMapVisibleEntities, RenderShadowMapVisibleEntities,
        },
    },
};

// Bevy 0.20's extract_lights removes shadow visibility for lights absent from
// its Changed<> query. On an idle extraction those lights are still visible.
// Retain the last render-world tables only when no application tick has run.
#[derive(Resource, Default)]
struct FrozenShadows(
    Vec<(
        Entity,
        RenderExtractedShadowMapVisibleEntities,
        RenderShadowMapVisibleEntities,
    )>,
);

#[derive(Resource)]
struct Installed(bool);

#[derive(Resource, Default)]
struct Ready(bool);

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Bootstrap;

pub(super) fn install(app: &mut App) {
    #[cfg(feature = "offscreen")]
    let pre_extract = app
        .world()
        .contains_resource::<super::super::offscreen::OffscreenTarget>()
        .then(|| {
            app.get_added_plugins::<bevy::render::extract_plugin::ExtractPlugin<RenderApp>>()
                .first()
                .map(|plugin| plugin.pre_extract)
        })
        .flatten();
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    let mut extract = render.take_extract();
    let schedule = render.update_schedule;
    render
        .init_resource::<Ready>()
        .init_resource::<FrozenShadows>()
        .add_systems(
            Render,
            restore_shadows
                .after(RenderSystems::ExtractCommands)
                .before(RenderSystems::PrepareAssets),
        );
    let mut last_tick = None;
    render.set_extract(move |main, render| {
        let ready = main.resource::<Installed>().0;
        render.resource_mut::<Ready>().0 = ready;
        if ready && let Some(extract) = extract.as_mut() {
            #[cfg(feature = "offscreen")]
            if let Some(pre_extract) = pre_extract {
                super::super::offscreen::protect(
                    main.get_resource::<super::super::offscreen::Status>()
                        .cloned(),
                    || pre_extract(main, render),
                );
                if main
                    .get_resource::<super::super::offscreen::Status>()
                    .is_some_and(|s| s.error().is_some())
                {
                    render.resource_mut::<Ready>().0 = false;
                    return;
                }
                super::super::offscreen::shaders(main, render);
            }
            let tick = main.resource::<super::Bridge>().ticks;
            let shadows = if last_tick == Some(tick) {
                render
                    .iter_entities()
                    .filter_map(|entity| {
                        Some((
                            entity.id(),
                            RenderExtractedShadowMapVisibleEntities {
                                subviews: entity
                                    .get::<RenderExtractedShadowMapVisibleEntities>()?
                                    .subviews
                                    .clone(),
                            },
                            entity.get::<RenderShadowMapVisibleEntities>()?.clone(),
                        ))
                    })
                    .collect()
            } else {
                Vec::new()
            };
            render.resource_mut::<FrozenShadows>().0 = shadows;
            last_tick = Some(tick);
            super::super::throughput::begin_extract(main);
            extract(main, render);
            super::super::throughput::end_extract(main);
        }
    });
    render.add_systems(Bootstrap, move |world: &mut World| {
        if world.resource::<Ready>().0
            && let Some(schedule) = schedule
        {
            #[cfg(feature = "offscreen")]
            super::super::offscreen::protect(
                world
                    .get_resource::<super::super::offscreen::Status>()
                    .cloned(),
                || world.run_schedule(schedule),
            );
            #[cfg(not(feature = "offscreen"))]
            world.run_schedule(schedule);
        }
    });
    render.update_schedule = Some(Bootstrap.intern());
    app.insert_resource(Installed(false));
}

fn restore_shadows(world: &mut World) {
    let shadows = std::mem::take(&mut world.resource_mut::<FrozenShadows>().0);
    for (entity, extracted, visible) in shadows {
        if let Ok(mut entity) = world.get_entity_mut(entity)
            && !entity.contains::<RenderShadowMapVisibleEntities>()
        {
            entity.insert((extracted, visible));
        }
    }
}

pub(super) fn start(world: &mut World) {
    if let Some(mut installed) = world.get_resource_mut::<Installed>() {
        installed.0 = true;
    }
}

#[cfg(feature = "screenshot")]
pub(super) fn waiting(world: &World) -> bool {
    world
        .get_resource::<Installed>()
        .is_some_and(|installed| !installed.0)
}
