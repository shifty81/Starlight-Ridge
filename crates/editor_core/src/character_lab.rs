#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterBaseBody {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterLayerKind {
    Body,
    Underwear,
    Hair,
    Eyes,
    Face,
    Shirt,
    Pants,
    Shoes,
    Accessory,
    Tool,
    Equipment,
}

#[derive(Debug, Clone)]
pub struct CharacterLayerSlot {
    pub id: &'static str,
    pub title: &'static str,
    pub kind: CharacterLayerKind,
    pub destructive: bool,
}

#[derive(Debug, Clone)]
pub struct CharacterLabDescriptor {
    pub male_base_path: &'static str,
    pub female_base_path: &'static str,
    pub frame_width: u32,
    pub frame_height: u32,
    pub source_layers_are_non_destructive: bool,
    pub layer_slots: Vec<CharacterLayerSlot>,
}

impl CharacterLabDescriptor {
    pub fn phase27_default() -> Self {
        Self {
            male_base_path: "assets/textures/characters/base_male_underwear_phase27.png",
            female_base_path: "assets/textures/characters/base_female_underwear_phase27.png",
            frame_width: 32,
            frame_height: 32,
            source_layers_are_non_destructive: true,
            layer_slots: vec![
                CharacterLayerSlot { id: "body", title: "Body", kind: CharacterLayerKind::Body, destructive: false },
                CharacterLayerSlot { id: "underwear", title: "Underwear", kind: CharacterLayerKind::Underwear, destructive: false },
                CharacterLayerSlot { id: "hair", title: "Hair", kind: CharacterLayerKind::Hair, destructive: false },
                CharacterLayerSlot { id: "eyes", title: "Eyes", kind: CharacterLayerKind::Eyes, destructive: false },
                CharacterLayerSlot { id: "face", title: "Face", kind: CharacterLayerKind::Face, destructive: false },
                CharacterLayerSlot { id: "shirt", title: "Shirt", kind: CharacterLayerKind::Shirt, destructive: false },
                CharacterLayerSlot { id: "pants", title: "Pants", kind: CharacterLayerKind::Pants, destructive: false },
                CharacterLayerSlot { id: "shoes", title: "Shoes", kind: CharacterLayerKind::Shoes, destructive: false },
                CharacterLayerSlot { id: "accessory", title: "Accessory", kind: CharacterLayerKind::Accessory, destructive: false },
                CharacterLayerSlot { id: "tool", title: "Tool", kind: CharacterLayerKind::Tool, destructive: false },
                CharacterLayerSlot { id: "equipment", title: "Equipment / R.I.G.", kind: CharacterLayerKind::Equipment, destructive: false },
            ],
        }
    }
}
