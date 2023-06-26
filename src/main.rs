use bevy::prelude::*;

use hover_pill::GamePlugin;

fn main() {
    let mut app = App::new();
    app.add_plugin(GamePlugin);
    app.run();
}
