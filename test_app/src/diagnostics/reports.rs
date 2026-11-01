//! Opt-in report probes. F9 acts only on an explicitly controlled update.
use bevy::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct ReportProbe {
    mode: String,
    actions: u64,
    caught: u64,
    handled_errors: u64,
}

pub(super) fn install(app: &mut App, mode: String) {
    assert!(matches!(
        mode.as_str(),
        "caught-system"
            | "caught-task"
            | "caught-worker"
            | "caught-any"
            | "handled-error"
            | "exit-0"
            | "exit-17"
            | "event-overflow"
    ));
    app.insert_resource(ReportProbe {
        mode,
        actions: 0,
        caught: 0,
        handled_errors: 0,
    })
    .register_type::<ReportProbe>()
    .add_systems(
        Update,
        trigger.in_set(crate::app::schedule::Phase::WorldActions),
    );
}

fn trigger(keys: Res<ButtonInput<KeyCode>>, mut probe: ResMut<ReportProbe>) {
    if !keys.just_pressed(KeyCode::F9) {
        return;
    }
    probe.actions += 1;
    let caught = match probe.mode.as_str() {
        "caught-system" => {
            std::panic::catch_unwind(|| panic!("Signalstation caught system panic")).is_err()
        }
        "caught-task" => {
            let task = bevy::tasks::AsyncComputeTaskPool::get().spawn(async {
                std::panic::catch_unwind(|| panic!("Signalstation caught task panic")).is_err()
            });
            bevy::tasks::block_on(task)
        }
        "caught-worker" => std::thread::spawn(|| panic!("Signalstation caught worker panic"))
            .join()
            .is_err(),
        "caught-any" => std::panic::catch_unwind(|| std::panic::panic_any(47_u32)).is_err(),
        "event-overflow" => {
            for index in 0..300 {
                error!(target: "signalstation::overflow", "Signalstation overflow event {index}");
            }
            false
        }
        "handled-error" => {
            let result: Result<(), &str> = Err("Signalstation handled error");
            probe.handled_errors += u64::from(result.is_err());
            false
        }
        "exit-0" | "exit-17" => {
            let code = if probe.mode == "exit-0" { 0 } else { 17 };
            eprintln!("Signalstation report probe exiting {code}");
            std::process::exit(code);
        }
        _ => unreachable!(),
    };
    probe.caught += u64::from(caught);
}
