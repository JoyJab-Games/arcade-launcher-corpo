use godot::prelude::*;

struct LauncherExtension;

#[gdextension]
unsafe impl ExtensionLibrary for LauncherExtension {}

// Beispiel: eine Klasse, die in Godot als Node auftaucht
#[derive(GodotClass)]
#[class(base=Node)]
struct SystemBridge {
    base: Base<Node>,
}

#[godot_api]
impl INode for SystemBridge {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl SystemBridge {
    #[func]
    fn scan_wifi(&self) -> Array<GString> {
        // hier später: zbus → NetworkManager
        array!["Testnetz"]
    }

    #[signal]
    fn sd_card_mounted(path: GString);
}
