from pathlib import Path
import shutil, struct, zipfile, os, math
from PIL import Image

ROOT = Path('/mnt/data/starlight_phase53_src')
assert ROOT.exists()

# ---------- helpers ----------
def ensure(p):
    p.mkdir(parents=True, exist_ok=True)
    return p

def write_text(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.strip()+"\n", encoding='utf-8')

def pack_chunk(cid, content=b'', children=b''):
    return cid.encode('ascii') + struct.pack('<II', len(content), len(children)) + content + children

def write_vox(path, size, voxels, palette):
    """Write a simple MagicaVoxel .vox v150 file.
    voxels: iterable (x,y,z,color_index), color_index 1..255
    palette: list of 256 RGBA tuples. palette[0] is index 1 in MagicaVoxel RGBA chunk.
    """
    x,y,z = size
    # unique, bounded, skip invalid
    seen = {}
    for vx,vy,vz,ci in voxels:
        if 0 <= vx < x and 0 <= vy < y and 0 <= vz < z and 1 <= ci <= 255:
            seen[(int(vx), int(vy), int(vz))] = int(ci)
    v = [(a,b,c,ci) for (a,b,c),ci in seen.items()]
    size_chunk = pack_chunk('SIZE', struct.pack('<III', x,y,z))
    xyzi_content = struct.pack('<I', len(v)) + b''.join(struct.pack('BBBB', vx,vy,vz,ci) for vx,vy,vz,ci in v)
    xyzi_chunk = pack_chunk('XYZI', xyzi_content)
    pal = list(palette[:256]) + [(0,0,0,0)]*(256-len(palette))
    rgba_chunk = pack_chunk('RGBA', b''.join(struct.pack('BBBB', r,g,b,a) for r,g,b,a in pal[:256]))
    children = size_chunk + xyzi_chunk + rgba_chunk
    data = b'VOX ' + struct.pack('<I',150) + pack_chunk('MAIN', b'', children)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return len(v)

# palette: indices are 1-based, pal[0] -> index 1
palette = [(0,0,0,0)] * 256
# reserved colors
colors = {
    'skin_light': (222, 174, 132, 255),
    'skin_mid': (202, 144, 100, 255),
    'skin_shadow': (166, 101, 72, 255),
    'skin_deep': (124, 72, 54, 255),
    'template_mark': (90, 170, 255, 255),
    'under': (92, 124, 150, 255),
    'sole': (90, 65, 52, 255),
    'metal': (160, 168, 172, 255),
    'metal_dark': (92, 100, 104, 255),
    'wood': (132, 82, 42, 255),
    'wood_dark': (86, 52, 30, 255),
    'watercan': (82, 148, 172, 255),
    'watercan_dark': (44, 94, 120, 255),
    'edge': (48, 54, 58, 255),
    'cloth': (110, 112, 128, 255),
}
idx = {}
for i,(name,rgba) in enumerate(colors.items(), start=1):
    idx[name] = i
    palette[i-1] = rgba

# ---------- voxel shape utilities ----------
def ellipsoid(cx, cy, cz, rx, ry, rz, ci, shade=True):
    vox=[]
    for x in range(math.floor(cx-rx)-1, math.ceil(cx+rx)+2):
        for y in range(math.floor(cy-ry)-1, math.ceil(cy+ry)+2):
            for z in range(math.floor(cz-rz)-1, math.ceil(cz+rz)+2):
                if ((x-cx)/rx)**2 + ((y-cy)/ry)**2 + ((z-cz)/rz)**2 <= 1.0:
                    c=ci
                    if shade:
                        if y < cy-ry*0.35 or x < cx-rx*0.52:
                            c = idx.get('skin_shadow', ci) if ci in (idx['skin_light'],idx['skin_mid']) else ci
                        if z < cz-rz*0.72:
                            c = idx.get('skin_mid', ci) if ci == idx['skin_light'] else c
                    vox.append((x,y,z,c))
    return vox

def box(x0,x1,y0,y1,z0,z1,ci):
    return [(x,y,z,ci) for x in range(x0,x1+1) for y in range(y0,y1+1) for z in range(z0,z1+1)]

def tapered_body(cx, cy, z0, z1, rx0, rx1, ry0, ry1, ci):
    vox=[]
    h=max(1,z1-z0)
    for z in range(z0,z1+1):
        t=(z-z0)/h
        rx=rx0*(1-t)+rx1*t
        ry=ry0*(1-t)+ry1*t
        for x in range(math.floor(cx-rx)-1, math.ceil(cx+rx)+2):
            for y in range(math.floor(cy-ry)-1, math.ceil(cy+ry)+2):
                if ((x-cx)/rx)**2 + ((y-cy)/ry)**2 <= 1.0:
                    c=ci
                    if x < cx-rx*0.45 or y < cy-ry*0.4: c=idx['skin_shadow']
                    vox.append((x,y,z,c))
    return vox

def limb_between(x0,y0,z0,x1,y1,z1,r,ci):
    vox=[]
    steps=max(abs(x1-x0), abs(y1-y0), abs(z1-z0), 1)
    for i in range(steps+1):
        t=i/steps
        cx=x0*(1-t)+x1*t
        cy=y0*(1-t)+y1*t
        cz=z0*(1-t)+z1*t
        vox += ellipsoid(cx,cy,cz,r,r*0.85,r*1.12,ci,shade=True)
    return vox

def make_character(kind='adult_a'):
    vox=[]
    if kind=='adult_a':
        size=(36,24,72); cx=18; cy=12; skin=idx['skin_light']
        # head, neck, torso
        vox += ellipsoid(cx,cy,60,7.1,6.4,8.8,skin)
        vox += box(cx-3,cx+3,cy-2,cy+2,48,53,skin)
        vox += tapered_body(cx,cy,31,50,6.2,8.1,4.9,5.6,skin)
        # waist/undergarment neutral template bands
        vox += box(cx-6,cx+6,cy-4,cy+4,29,34,idx['under'])
        # arms slightly bent downward
        vox += limb_between(cx-8,cy,44,cx-13,cy,34,2.7,skin)
        vox += limb_between(cx+8,cy,44,cx+13,cy,34,2.7,skin)
        vox += limb_between(cx-13,cy,33,cx-12,cy,22,2.5,skin)
        vox += limb_between(cx+13,cy,33,cx+12,cy,22,2.5,skin)
        vox += ellipsoid(cx-12,cy,19,2.8,2.4,2.6,skin)
        vox += ellipsoid(cx+12,cy,19,2.8,2.4,2.6,skin)
        # legs
        vox += limb_between(cx-4,cy,30,cx-5,cy,16,3.0,skin)
        vox += limb_between(cx+4,cy,30,cx+5,cy,16,3.0,skin)
        vox += limb_between(cx-5,cy,15,cx-5,cy,5,2.7,skin)
        vox += limb_between(cx+5,cy,15,cx+5,cy,5,2.7,skin)
        vox += box(cx-8,cx-3,cy-5,cy+3,1,4,idx['sole'])
        vox += box(cx+3,cx+8,cy-5,cy+3,1,4,idx['sole'])
        # template reference markers: joint dots on back/side, not face/hair
        vox += box(cx-1,cx+1,cy+5,cy+5,42,44,idx['template_mark'])
        vox += box(cx-13,cx-13,cy+3,cy+3,32,34,idx['template_mark'])
        vox += box(cx+13,cx+13,cy+3,cy+3,32,34,idx['template_mark'])
    else:
        size=(34,22,68); cx=17; cy=11; skin=idx['skin_mid']
        vox += ellipsoid(cx,cy,57,6.6,5.8,8.2,skin)
        vox += box(cx-3,cx+3,cy-2,cy+2,46,51,skin)
        vox += tapered_body(cx,cy,29,48,5.7,7.2,4.3,5.2,skin)
        vox += box(cx-5,cx+5,cy-4,cy+4,27,32,idx['cloth'])
        vox += limb_between(cx-7,cy,42,cx-12,cy,32,2.5,skin)
        vox += limb_between(cx+7,cy,42,cx+12,cy,32,2.5,skin)
        vox += limb_between(cx-12,cy,31,cx-11,cy,21,2.3,skin)
        vox += limb_between(cx+12,cy,31,cx+11,cy,21,2.3,skin)
        vox += ellipsoid(cx-11,cy,18,2.5,2.1,2.4,skin)
        vox += ellipsoid(cx+11,cy,18,2.5,2.1,2.4,skin)
        vox += limb_between(cx-4,cy,28,cx-4,cy,15,2.7,skin)
        vox += limb_between(cx+4,cy,28,cx+4,cy,15,2.7,skin)
        vox += limb_between(cx-4,cy,14,cx-4,cy,5,2.4,skin)
        vox += limb_between(cx+4,cy,14,cx+4,cy,5,2.4,skin)
        vox += box(cx-7,cx-2,cy-4,cy+3,1,4,idx['sole'])
        vox += box(cx+2,cx+7,cy-4,cy+3,1,4,idx['sole'])
        vox += box(cx-1,cx+1,cy+5,cy+5,40,42,idx['template_mark'])
    return size, vox

def cylinder_line(x0,y0,z0,x1,y1,z1,r,ci):
    return limb_between(x0,y0,z0,x1,y1,z1,r,ci)

def make_hoe():
    size=(76,20,10); vox=[]
    vox += cylinder_line(6,10,5,58,10,5,1.25,idx['wood'])
    vox += cylinder_line(6,10,4,58,10,4,0.75,idx['wood_dark'])
    vox += box(56,64,8,12,3,7,idx['metal_dark'])
    vox += box(64,72,5,15,2,8,idx['metal'])
    vox += box(70,74,4,16,1,9,idx['edge'])
    return size, vox

def make_axe():
    size=(72,24,12); vox=[]
    vox += cylinder_line(8,12,6,54,12,6,1.35,idx['wood'])
    vox += cylinder_line(8,12,5,54,12,5,0.75,idx['wood_dark'])
    vox += box(50,58,10,14,3,9,idx['metal_dark'])
    # blade crescent-ish
    for x in range(55,70):
        for y in range(5,20):
            for z in range(2,11):
                if ((x-58)/11)**2 + ((y-12)/8)**2 + ((z-6)/5)**2 <= 1 and x>57:
                    vox.append((x,y,z,idx['metal'] if x<67 else idx['edge']))
    return size, vox

def make_pickaxe():
    size=(80,24,12); vox=[]
    vox += cylinder_line(10,12,6,58,12,6,1.3,idx['wood'])
    vox += cylinder_line(10,12,5,58,12,5,0.75,idx['wood_dark'])
    vox += box(54,62,10,14,3,9,idx['metal_dark'])
    vox += cylinder_line(56,12,7,74,5,7,1.2,idx['metal'])
    vox += cylinder_line(56,12,7,74,19,7,1.2,idx['metal'])
    vox += box(72,76,3,7,5,9,idx['edge'])
    vox += box(72,76,17,21,5,9,idx['edge'])
    return size, vox

def make_watering_can():
    size=(48,32,24); vox=[]
    # body ellipsoid/cuboid detail
    vox += ellipsoid(21,16,10,12,9,8,idx['watercan'],shade=False)
    vox += box(11,31,8,24,6,16,idx['watercan'])
    # darker lower/back shade
    vox += box(11,31,8,10,5,14,idx['watercan_dark'])
    # handle arch
    vox += cylinder_line(16,24,14,24,28,20,1.2,idx['watercan_dark'])
    vox += cylinder_line(24,28,20,31,23,14,1.2,idx['watercan_dark'])
    # spout
    vox += cylinder_line(31,16,13,43,14,16,1.35,idx['watercan'])
    vox += box(41,45,11,16,14,18,idx['edge'])
    return size, vox

def make_sword():
    size=(72,16,10); vox=[]
    vox += box(4,18,7,9,4,6,idx['wood'])
    vox += box(17,22,4,12,3,7,idx['metal_dark'])
    vox += cylinder_line(22,8,5,63,8,5,1.25,idx['metal'])
    vox += box(62,69,7,9,4,6,idx['edge'])
    return size, vox

vox_dir = ROOT/'content/voxels'
base_dir = ensure(vox_dir/'characters/base_templates')
tool_dir = ensure(vox_dir/'tools/base_templates')
counts={}
for name,kind in [('character_base_template_adult_a_bald_clean.vox','adult_a'),('character_base_template_adult_b_bald_clean.vox','adult_b')]:
    size, vox = make_character(kind)
    counts[name]=write_vox(base_dir/name,size,vox,palette)
for name,fn in [
    ('tool_hoe_high_detail_template.vox', make_hoe),
    ('tool_axe_high_detail_template.vox', make_axe),
    ('tool_pickaxe_high_detail_template.vox', make_pickaxe),
    ('tool_watering_can_high_detail_template.vox', make_watering_can),
    ('tool_sword_high_detail_template.vox', make_sword),
]:
    size, vox = fn()
    counts[name]=write_vox(tool_dir/name,size,vox,palette)

# Copy user source refs
refs_dir = ensure(ROOT/'assets/art_source/phase53_character_refs')
ref_sources = [
    ('34266456.PNG','reference_character_01.png'),
    ('124132515.PNG','reference_character_02.png'),
    ('456456.PNG','reference_character_03.png'),
    ('111.PNG','reference_character_04.png'),
]
for src,dst in ref_sources:
    sp = Path('/mnt/data')/src
    if sp.exists():
        im=Image.open(sp)
        im.save(refs_dir/dst)

# Content contracts
ensure(ROOT/'content/voxel_assets')
write_text(ROOT/'content/voxel_assets/voxel_asset_registry.ron', r'''
VoxelAssetRegistry(
    phase: "phase53_pixel_voxel_first_master_spec",
    default_density_profile: HeroDetail,
    default_voxels_per_tile: 64,
    assets: [
        VoxelAssetDef(
            id: "character_base_template_adult_a_bald_clean",
            category: Character,
            source_path: "content/voxels/characters/base_templates/character_base_template_adult_a_bald_clean.vox",
            display_name: "Adult Character Base Template A - Bald/Clean Face",
            tags: ["character", "npc", "player", "base_template", "bald", "clean_face", "pixel_voxel", "hero_detail"],
            material_profile: "pixel_voxel_character_base",
            density_profile: HeroDetail,
            voxels_per_tile: 64,
            max_voxel_budget: 65536,
            pivot: PivotDef(mode: FeetCenter, offset: [0.0, 0.0, 0.0]),
            scale: 1.0,
            collision: CapsuleApprox,
            render_mode: RuntimeVoxelOrBakedSprite,
            bake_profile: Some("character_8dir_pixel_voxel_64"),
            source_refs: ["assets/art_source/phase53_character_refs/reference_character_01.png", "assets/art_source/phase53_character_refs/reference_character_02.png"],
        ),
        VoxelAssetDef(
            id: "character_base_template_adult_b_bald_clean",
            category: Character,
            source_path: "content/voxels/characters/base_templates/character_base_template_adult_b_bald_clean.vox",
            display_name: "Adult Character Base Template B - Bald/Clean Face",
            tags: ["character", "npc", "player", "base_template", "bald", "clean_face", "pixel_voxel", "hero_detail"],
            material_profile: "pixel_voxel_character_base",
            density_profile: HeroDetail,
            voxels_per_tile: 64,
            max_voxel_budget: 65536,
            pivot: PivotDef(mode: FeetCenter, offset: [0.0, 0.0, 0.0]),
            scale: 1.0,
            collision: CapsuleApprox,
            render_mode: RuntimeVoxelOrBakedSprite,
            bake_profile: Some("character_8dir_pixel_voxel_64"),
            source_refs: ["assets/art_source/phase53_character_refs/reference_character_03.png", "assets/art_source/phase53_character_refs/reference_character_04.png"],
        ),
        VoxelAssetDef(
            id: "tool_hoe_high_detail_template",
            category: Tool,
            source_path: "content/voxels/tools/base_templates/tool_hoe_high_detail_template.vox",
            display_name: "High Detail Hoe Template",
            tags: ["tool", "hoe", "pixel_voxel", "hero_detail", "right_hand_attachment"],
            material_profile: "pixel_voxel_tool_base",
            density_profile: HeroDetail,
            voxels_per_tile: 64,
            max_voxel_budget: 32768,
            pivot: PivotDef(mode: GripPoint, offset: [6.0, 10.0, 5.0]),
            scale: 1.0,
            collision: Bounds,
            render_mode: RuntimeVoxelOrBakedSprite,
            bake_profile: Some("held_tool_pixel_voxel_64"),
            source_refs: [],
        ),
        VoxelAssetDef(
            id: "tool_axe_high_detail_template",
            category: Tool,
            source_path: "content/voxels/tools/base_templates/tool_axe_high_detail_template.vox",
            display_name: "High Detail Axe Template",
            tags: ["tool", "axe", "pixel_voxel", "hero_detail", "right_hand_attachment"],
            material_profile: "pixel_voxel_tool_base",
            density_profile: HeroDetail,
            voxels_per_tile: 64,
            max_voxel_budget: 32768,
            pivot: PivotDef(mode: GripPoint, offset: [8.0, 12.0, 6.0]),
            scale: 1.0,
            collision: Bounds,
            render_mode: RuntimeVoxelOrBakedSprite,
            bake_profile: Some("held_tool_pixel_voxel_64"),
            source_refs: [],
        ),
        VoxelAssetDef(
            id: "tool_pickaxe_high_detail_template",
            category: Tool,
            source_path: "content/voxels/tools/base_templates/tool_pickaxe_high_detail_template.vox",
            display_name: "High Detail Pickaxe Template",
            tags: ["tool", "pickaxe", "pixel_voxel", "hero_detail", "right_hand_attachment"],
            material_profile: "pixel_voxel_tool_base",
            density_profile: HeroDetail,
            voxels_per_tile: 64,
            max_voxel_budget: 32768,
            pivot: PivotDef(mode: GripPoint, offset: [10.0, 12.0, 6.0]),
            scale: 1.0,
            collision: Bounds,
            render_mode: RuntimeVoxelOrBakedSprite,
            bake_profile: Some("held_tool_pixel_voxel_64"),
            source_refs: [],
        ),
        VoxelAssetDef(
            id: "tool_watering_can_high_detail_template",
            category: Tool,
            source_path: "content/voxels/tools/base_templates/tool_watering_can_high_detail_template.vox",
            display_name: "High Detail Watering Can Template",
            tags: ["tool", "watering_can", "pixel_voxel", "hero_detail", "right_hand_attachment"],
            material_profile: "pixel_voxel_tool_base",
            density_profile: HeroDetail,
            voxels_per_tile: 64,
            max_voxel_budget: 32768,
            pivot: PivotDef(mode: GripPoint, offset: [18.0, 25.0, 16.0]),
            scale: 1.0,
            collision: Bounds,
            render_mode: RuntimeVoxelOrBakedSprite,
            bake_profile: Some("held_tool_pixel_voxel_64"),
            source_refs: [],
        ),
        VoxelAssetDef(
            id: "tool_sword_high_detail_template",
            category: Tool,
            source_path: "content/voxels/tools/base_templates/tool_sword_high_detail_template.vox",
            display_name: "High Detail Sword Template",
            tags: ["tool", "sword", "combat", "pixel_voxel", "hero_detail", "right_hand_attachment"],
            material_profile: "pixel_voxel_tool_base",
            density_profile: HeroDetail,
            voxels_per_tile: 64,
            max_voxel_budget: 32768,
            pivot: PivotDef(mode: GripPoint, offset: [5.0, 8.0, 5.0]),
            scale: 1.0,
            collision: Bounds,
            render_mode: RuntimeVoxelOrBakedSprite,
            bake_profile: Some("held_tool_pixel_voxel_64"),
            source_refs: [],
        ),
    ],
)
''')

write_text(ROOT/'content/voxel_assets/voxel_materials.ron', r'''
VoxelMaterialRegistry(
    phase: "phase53_pixel_voxel_first_master_spec",
    materials: [
        VoxelMaterialDef(id: "skin_light", display_name: "Base Skin Light", tags: ["character", "skin", "base_template"], solid: true, destructible: false, tool_tags: [], footstep_sound: None, impact_sound: Some("soft_hit"), particle: None, seasonal_variants: []),
        VoxelMaterialDef(id: "skin_mid", display_name: "Base Skin Mid", tags: ["character", "skin", "base_template"], solid: true, destructible: false, tool_tags: [], footstep_sound: None, impact_sound: Some("soft_hit"), particle: None, seasonal_variants: []),
        VoxelMaterialDef(id: "template_marker_blue", display_name: "Template Joint Marker", tags: ["editor", "marker", "attachment"], solid: false, destructible: false, tool_tags: [], footstep_sound: None, impact_sound: None, particle: None, seasonal_variants: []),
        VoxelMaterialDef(id: "tool_wood", display_name: "Tool Wood", tags: ["tool", "wood"], solid: true, destructible: true, tool_tags: ["axe"], footstep_sound: None, impact_sound: Some("wood_hit"), particle: Some("wood_chip"), seasonal_variants: []),
        VoxelMaterialDef(id: "tool_metal", display_name: "Tool Metal", tags: ["tool", "metal"], solid: true, destructible: true, tool_tags: ["hammer"], footstep_sound: None, impact_sound: Some("metal_hit"), particle: Some("spark"), seasonal_variants: []),
    ],
)
''')

write_text(ROOT/'content/voxel_assets/voxel_import_rules.ron', r'''
VoxelImportRules(
    phase: "phase53_pixel_voxel_first_master_spec",
    rules: [
        VoxelImportRule(source_glob: "content/voxels/characters/**/*.vox", default_category: Character, infer_category_from_folder: true, auto_register: false, default_density_profile: HeroDetail, default_voxels_per_tile: 64, default_collision: CapsuleApprox, default_render_mode: RuntimeVoxelOrBakedSprite),
        VoxelImportRule(source_glob: "content/voxels/tools/**/*.vox", default_category: Tool, infer_category_from_folder: true, auto_register: false, default_density_profile: HeroDetail, default_voxels_per_tile: 64, default_collision: Bounds, default_render_mode: RuntimeVoxelOrBakedSprite),
        VoxelImportRule(source_glob: "content/voxels/buildings/**/*.vox", default_category: Building, infer_category_from_folder: true, auto_register: false, default_density_profile: PixelVoxel64, default_voxels_per_tile: 64, default_collision: BoxApprox, default_render_mode: RuntimeVoxelOrBakedSprite),
    ],
)
''')

write_text(ROOT/'content/voxel_assets/voxel_validation_rules.ron', r'''
VoxelValidationRules(
    phase: "phase53_pixel_voxel_first_master_spec",
    hard_errors: [
        "duplicate_voxel_asset_id",
        "registered_voxel_asset_missing_source_file",
        "invalid_voxel_asset_category",
        "invalid_density_profile",
        "invalid_voxels_per_tile_zero",
    ],
    warnings: [
        "character_base_has_hair_or_facial_hair_tag",
        "character_or_npc_under_hero_detail_density",
        "tool_under_hero_detail_density",
        "missing_bake_profile",
        "missing_source_reference_for_base_template",
        "unsupported_vox_chunk",
    ],
    character_rules: CharacterVoxelRules(
        require_bald_clean_base_templates: true,
        default_density_profile: HeroDetail,
        default_voxels_per_tile: 64,
        recommended_max_voxel_budget: 65536,
        forbid_base_template_tags: ["hair", "beard", "mustache", "facial_hair"],
    ),
    tool_rules: ToolVoxelRules(
        default_density_profile: HeroDetail,
        default_voxels_per_tile: 64,
        recommended_max_voxel_budget: 32768,
    ),
)
''')

# Scene placeholders
ensure(ROOT/'content/scenes/starter_farm')
write_text(ROOT/'content/scenes/starter_farm/voxel_scene.ron', r'''
VoxelSceneDef(
    scene_id: "starter_farm",
    terrain_ref: None,
    object_set_ref: Some("starter_farm_voxel_objects"),
    default_palette: "starlight_pixel_voxel_default",
    units_per_voxel: 0.03125,
    grid: VoxelSceneGridDef(
        tile_size_voxels: [64, 64, 16],
        chunk_size_tiles: [16, 16],
        world_up_axis: "Z",
    ),
)
''')
write_text(ROOT/'content/scenes/starter_farm/voxel_objects.ron', r'''
VoxelObjectSetDef(
    id: "starter_farm_voxel_objects",
    scene_id: "starter_farm",
    objects: [
        VoxelSceneObjectDef(
            id: "character_base_template_preview_a",
            asset_id: "character_base_template_adult_a_bald_clean",
            position: [4.0, 4.0, 0.0],
            rotation_degrees: [0.0, 0.0, 0.0],
            scale: 1.0,
            layer: "editor_preview",
            tags: ["preview", "character_base_template"],
            collision_enabled: false,
            interaction_id: None,
        ),
        VoxelSceneObjectDef(
            id: "character_base_template_preview_b",
            asset_id: "character_base_template_adult_b_bald_clean",
            position: [6.0, 4.0, 0.0],
            rotation_degrees: [0.0, 0.0, 0.0],
            scale: 1.0,
            layer: "editor_preview",
            tags: ["preview", "character_base_template"],
            collision_enabled: false,
            interaction_id: None,
        ),
    ],
)
''')
write_text(ROOT/'content/scenes/starter_farm/voxel_terrain.ron', r'''
VoxelTerrainDef(
    id: "starter_farm_voxel_terrain_phase53_placeholder",
    scene_id: "starter_farm",
    density_profile: PixelVoxel64,
    voxels_per_tile: 64,
    chunks: [],
)
''')

# Docs
write_text(ROOT/'docs/PHASE53_PIXEL_VOXEL_FIRST_MASTER_SPEC.md', r'''
# Phase 53 - Pixel-Voxel First Master Spec

Starlight Ridge is now a **pixel-voxel-first** project. Characters, NPCs, tools, buildings, props, foliage, terrain forms, furniture, machines, pickups, and world items should be authored from voxels as the source of truth.

The visual target is **micro-voxel / pixel-voxel detail**, not chunky block art. Runtime rendering may still use optimized meshes, baked sprites, impostors, or 2.5D presentation, but source assets are voxel-authored.

## Phase 53 lock-ins

- Characters and NPCs use the highest practical voxel density for readable farm/life-sim characters.
- Tools use high-detail voxel templates because they need readable silhouettes during tool-use animation.
- Base character templates must have no hair and no facial hair.
- Hair, beards, mustaches, hats, helmets, outfits, backpacks, and accessories become separate swappable voxel parts later.
- `.vox` is a first-class asset source format.
- All voxel editor tabs must be child panels only and must not render the full editor shell inside themselves.

## Density profiles

| Profile | Use | Default budget |
| --- | --- | --- |
| PixelVoxel16 | tiny/background/simple props | 4k voxels |
| PixelVoxel32 | general props/world objects | 16k voxels |
| PixelVoxel64 | buildings, terrain features, important props | 64k voxels |
| HeroDetail | characters, NPCs, animals, tools, interactables | 32k-65k voxels |

## Immediate content added

This patch adds starter `.vox` templates:

- `content/voxels/characters/base_templates/character_base_template_adult_a_bald_clean.vox`
- `content/voxels/characters/base_templates/character_base_template_adult_b_bald_clean.vox`
- `content/voxels/tools/base_templates/tool_hoe_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_axe_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_pickaxe_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_watering_can_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_sword_high_detail_template.vox`

These are base starter assets, not final art. They exist to lock the pipeline shape, density scale, pivot expectations, and asset registry structure.
''')

write_text(ROOT/'docs/VOXEL_CHARACTER_BASE_TEMPLATE_GUIDE.md', r'''
# Voxel Character Base Template Guide

## Base-template rule

Character base templates are body/mannequin sources only.

They must not include:

- hair
- beard
- mustache
- eyebrows as hair chunks
- facial hair
- hats
- helmets
- final outfits

They may include:

- neutral mannequin body volume
- simple skin/material tone
- minimal underlayer markers
- editor-only joint/attachment markers
- blank head shape
- hand/foot proportions

## Recommended scale

The current Phase 53 templates use high-detail pixel-voxel proportions:

- adult base height target: roughly 60-72 voxels
- default character density profile: `HeroDetail`
- default `voxels_per_tile`: `64`
- recommended maximum budget: 65,536 voxels per assembled character before optimization/bake

## Why bald/clean base templates

Hair, facial hair, hats, outfits, and accessories must become swappable voxel parts. This keeps character customization, NPC variation, seasonal outfits, and animation overrides manageable.

## Later character parts

Recommended future folders:

```txt
content/voxels/characters/parts/hair/
content/voxels/characters/parts/facial_hair/
content/voxels/characters/parts/outfits/
content/voxels/characters/parts/hats/
content/voxels/characters/parts/tools_held/
content/voxels/characters/parts/backpacks/
```
''')

write_text(ROOT/'docs/PHASE53_HIGH_DETAIL_CHARACTER_TOOL_BASE_TEMPLATES.md', f'''
# Phase 53 High-Detail Character and Tool Base Templates

This patch adds generated starter `.vox` files for the pixel-voxel-first direction.

## Character templates

Both character base templates are intentionally bald and clean-faced.

| Asset | Purpose | Approx. generated voxel count |
| --- | --- | ---: |
| `character_base_template_adult_a_bald_clean.vox` | adult player/NPC base body template A | {counts.get('character_base_template_adult_a_bald_clean.vox', 0)} |
| `character_base_template_adult_b_bald_clean.vox` | adult player/NPC base body template B | {counts.get('character_base_template_adult_b_bald_clean.vox', 0)} |

## Tool templates

| Asset | Purpose | Approx. generated voxel count |
| --- | --- | ---: |
| `tool_hoe_high_detail_template.vox` | held hoe template | {counts.get('tool_hoe_high_detail_template.vox', 0)} |
| `tool_axe_high_detail_template.vox` | held axe template | {counts.get('tool_axe_high_detail_template.vox', 0)} |
| `tool_pickaxe_high_detail_template.vox` | held pickaxe template | {counts.get('tool_pickaxe_high_detail_template.vox', 0)} |
| `tool_watering_can_high_detail_template.vox` | held watering can template | {counts.get('tool_watering_can_high_detail_template.vox', 0)} |
| `tool_sword_high_detail_template.vox` | held sword/combat template | {counts.get('tool_sword_high_detail_template.vox', 0)} |

## Source references

The uploaded character reference images were copied into:

```txt
assets/art_source/phase53_character_refs/
```

The current `.vox` files are not exact image-to-voxel conversions. They are clean base meshes inspired by the requested direction and prepared for later manual refinement in MagicaVoxel/Blockbench/Blender.
''')

write_text(ROOT/'docs/EXTERNAL_TOOLS_MAGICA_BLOCKBENCH_BLENDER.md', r'''
# External Tool Plan - MagicaVoxel, Blockbench, Blender

## MagicaVoxel

Primary voxel authoring tool for `.vox` assets.

Initial editor actions:

- detect MagicaVoxel path
- open selected `.vox`
- refresh after save
- validate `.vox` dimensions, palette, and missing files
- import unregistered `.vox` assets into the registry

## Blockbench

Secondary tool for block/voxel-style rig planning and animation blocking.

Initial editor actions:

- detect Blockbench path
- open supported source files
- export/import through configured content folders

Blockbench should not replace the `.vox` source pipeline; it is a companion tool.

## Blender

Advanced render/bake/preview tool.

Initial editor actions:

- detect Blender path
- run configured bake scripts
- generate thumbnails/turntables later
- assist with mesh cleanup, sprite bake, normal/depth bake, and promotional renders
''')

write_text(ROOT/'docs/VOXEL_SCENE_FORMAT.md', r'''
# Voxel Scene Format

Scene maps keep the existing gameplay/control layer files and gain voxel scene files beside them.

Recommended scene folder shape:

```txt
content/scenes/<scene_id>/
  voxel_scene.ron
  voxel_objects.ron
  voxel_terrain.ron
```

`voxel_scene.ron` defines grid/density rules. `voxel_objects.ron` defines placed voxel assets. `voxel_terrain.ron` defines scene terrain chunks.

Gameplay map layers remain useful for soil state, watered state, collision hints, placement masks, interactions, pathfinding, biome IDs, season rules, and validation overlays.
''')

write_text(ROOT/'content/voxels/characters/base_templates/README.md', r'''
# Character Base Templates

Phase 53 starter character bases live here.

Rules:

- base templates are bald
- base templates have no facial hair
- hair/facial hair/hats/outfits are separate voxel parts later
- default density is HeroDetail / 64 voxels per tile
- these are source-of-truth voxel assets, not decorative imports
''')

write_text(ROOT/'content/voxels/tools/base_templates/README.md', r'''
# High-Detail Tool Templates

Phase 53 starter tool templates live here.

Tools should use high-detail pixel-voxel silhouettes because they need to read clearly during held, swing, slash, chop, hoe, mine, water, and pickup animations.
''')

write_text(ROOT/'PATCH_MANIFEST_phase53_pixel_voxel_first.txt', f'''
Starlight_Ridge_phase53_pixel_voxel_first_character_tool_templates

Added:
- Pixel-voxel-first master docs.
- High-detail character base template docs.
- External tool plan for MagicaVoxel, Blockbench, and Blender.
- Voxel scene format docs.
- Voxel asset registry RON.
- Voxel material registry RON.
- Voxel import/validation rule RON.
- Starter scene voxel placeholder files.
- Two bald/clean-face high-detail character base `.vox` templates.
- Five high-detail held tool `.vox` templates.
- Copied uploaded character image references into assets/art_source/phase53_character_refs/.

Generated voxel counts:
{chr(10).join(f'- {k}: {v}' for k,v in counts.items())}

Notes:
- The `.vox` templates are generated foundation assets, not final art.
- No Rust code was changed in this patch, so it should not introduce compile errors.
- Future patch should wire the voxel registry/contracts into game_data and add egui child-panel tabs for 3D voxel preview without nested-editor regression.
''')

# Add generation script into repo
script_dir=ensure(ROOT/'tools/scripts/voxel')
shutil.copy(Path('/mnt/data/generate_phase53_patch.py'), script_dir/'generate_phase53_pixel_voxel_templates.py')

print('counts')
for k,v in counts.items(): print(k,v)
