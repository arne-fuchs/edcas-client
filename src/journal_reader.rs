use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use edcas_common::journal::{CarrierJump, FsdJump, JournalEvent, Location, Scan};
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct SystemData {
    pub name: String,
    pub system_address: i64,
    pub coords: (f32, f32, f32),
    pub economy: String,
    pub second_economy: String,
    pub government: String,
    pub allegiance: String,
    pub security: String,
    pub population: i64,
    pub body: String,
    pub body_id: i32,
    pub body_type: String,
    pub factions: Vec<String>,
    pub system_faction: String,
    pub controlling_power: Option<String>,
    pub powers: Vec<String>,
}

#[derive(Clone)]
pub struct BodyScan {
    pub body_id: i32,
    pub body_name: String,
    pub planet_class: String,
    pub landable: bool,
    pub scan_type: String,
    pub distance_from_arrival_ls: f32,
    pub radius: f32,
    pub mass_em: f32,
    pub surface_temperature: f32,
    pub surface_gravity: f32,
    pub tidal_lock: bool,
    pub volcanism: String,
    pub atmosphere: String,
    pub terraform_state: String,
    pub star_type: String,
    pub parents: Vec<BodyParent>,
    pub rings: Vec<BodyRing>,
    pub materials: Vec<BodyMaterial>,
    pub estimated_value: i64,
    pub composition: Option<BodyComposition>,
}

#[derive(Clone)]
pub struct BodyComposition {
    pub ice: f32,
    pub rock: f32,
    pub metal: f32,
}

#[derive(Clone)]
pub struct BodyParent {
    pub body_id: i32,
    pub parent_type: ParentType,
}

#[derive(Clone)]
pub enum ParentType {
    Star,
    Planet,
    Ring,
    Null,
}

#[derive(Clone)]
pub struct BodyRing {
    pub name: String,
    pub ring_class: String,
    pub mass_mt: f64,
    pub inner_rad: f64,
    pub outer_rad: f64,
}

#[derive(Clone)]
pub struct BodyMaterial {
    pub name: String,
    pub percent: f64,
}

#[derive(Clone)]
pub struct JournalData {
    pub current_system: Option<SystemData>,
    pub bodies: Vec<BodyScan>,
}

impl JournalData {
    pub fn new() -> Self {
        Self {
            current_system: None,
            bodies: Vec::new(),
        }
    }

    pub fn process_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        let value: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => return,
        };

        match JournalEvent::from_json(value) {
            Some(JournalEvent::FsdJump(e)) => {
                debug!("FSDJump to {}", e.star_system);
                self.current_system = Some(system_from_fsdjump(&e));
                self.bodies.clear();
            }
            Some(JournalEvent::Location(e)) => {
                debug!("Location: {}", e.star_system);
                self.current_system = Some(system_from_location(&e));
                self.bodies.clear();
            }
            Some(JournalEvent::CarrierJump(e)) => {
                debug!("CarrierJump to {}", e.star_system);
                self.current_system = Some(system_from_carrier_jump(&e));
                self.bodies.clear();
            }
            Some(JournalEvent::Scan(e)) => {
                debug!("Scan: {}", e.body_name);
                self.bodies.push(body_from_scan(&e));
            }
            Some(JournalEvent::ScanBaryCentre(e)) => {
                // Barycentre has no useful display data; add a placeholder so the tree renders
                self.bodies.push(BodyScan {
                    body_id: e.body_id,
                    body_name: format!("{} Barycentre", e.star_system),
                    planet_class: "Barycentre".into(),
                    landable: false,
                    scan_type: "AutoScan".into(),
                    distance_from_arrival_ls: e.distance_from_arrival_ls,
                    radius: 0.0,
                    mass_em: 0.0,
                    surface_temperature: 0.0,
                    surface_gravity: 0.0,
                    tidal_lock: false,
                    volcanism: String::new(),
                    atmosphere: String::new(),
                    terraform_state: String::new(),
                    star_type: String::new(),
                    parents: vec![],
                    rings: vec![],
                    materials: vec![],
                    estimated_value: 0,
                    composition: None,
                });
            }
            _ => {}
        }
    }

    pub fn clear(&mut self) {
        self.current_system = None;
        self.bodies.clear();
    }
}

fn system_from_fsdjump(e: &FsdJump) -> SystemData {
    let coords = coords_from_star_pos(&e.star_pos);
    SystemData {
        name: e.star_system.clone(),
        system_address: e.system_address,
        coords,
        economy: e.system_economy.clone(),
        second_economy: e.system_second_economy.clone(),
        government: e.system_government.clone(),
        allegiance: e.system_allegiance.clone(),
        security: e.system_security.clone(),
        population: e.population,
        body: e.body.clone(),
        body_id: e.body_id,
        body_type: e.body_type.clone(),
        factions: e
            .factions
            .as_ref()
            .map(|f| f.iter().map(|x| x.name.clone()).collect())
            .unwrap_or_default(),
        system_faction: e
            .system_faction
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default(),
        controlling_power: e.controlling_power.clone(),
        powers: e.powers.clone().unwrap_or_default(),
    }
}

fn system_from_location(e: &Location) -> SystemData {
    let coords = coords_from_star_pos(&e.star_pos);
    SystemData {
        name: e.star_system.clone(),
        system_address: e.system_address,
        coords,
        economy: e.system_economy.clone(),
        second_economy: e.system_second_economy.clone(),
        government: e.system_government.clone(),
        allegiance: e.system_allegiance.clone(),
        security: e.system_security.clone(),
        population: e.population,
        body: e.body.clone(),
        body_id: e.body_id as i32,
        body_type: e.body_type.clone(),
        factions: e
            .factions
            .as_ref()
            .map(|f| f.iter().map(|x| x.name.clone()).collect())
            .unwrap_or_default(),
        system_faction: e
            .system_faction
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default(),
        controlling_power: e.controlling_power.clone(),
        powers: e.powers.clone().unwrap_or_default(),
    }
}

fn system_from_carrier_jump(e: &CarrierJump) -> SystemData {
    let coords = coords_from_star_pos(&e.star_pos);
    SystemData {
        name: e.star_system.clone(),
        system_address: e.system_address,
        coords,
        economy: e.system_economy.clone(),
        second_economy: e.system_second_economy.clone(),
        government: e.system_government.clone(),
        allegiance: e.system_allegiance.clone(),
        security: e.system_security.clone(),
        population: e.population,
        body: e.body.clone(),
        body_id: e.body_id,
        body_type: e.body_type.clone(),
        factions: e
            .factions
            .as_ref()
            .map(|f| f.iter().map(|x| x.name.clone()).collect())
            .unwrap_or_default(),
        system_faction: e
            .system_faction
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default(),
        controlling_power: e.controlling_power.clone(),
        powers: e.powers.clone().unwrap_or_default(),
    }
}

fn coords_from_star_pos(star_pos: &[f32]) -> (f32, f32, f32) {
    (
        star_pos.first().copied().unwrap_or(0.0),
        star_pos.get(1).copied().unwrap_or(0.0),
        star_pos.get(2).copied().unwrap_or(0.0),
    )
}

fn body_from_scan(e: &Scan) -> BodyScan {
    let parents = e
        .parents
        .as_ref()
        .map(|pv| {
            pv.iter()
                .filter_map(|p| {
                    p.parent_id().map(|pid| BodyParent {
                        body_id: pid,
                        parent_type: match p.parent_type() {
                            "Star" => ParentType::Star,
                            "Planet" => ParentType::Planet,
                            "Ring" => ParentType::Ring,
                            _ => ParentType::Null,
                        },
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let rings = e
        .rings
        .as_ref()
        .map(|rv| {
            rv.iter()
                .map(|r| BodyRing {
                    name: r.name.clone(),
                    ring_class: r.ring_class.clone(),
                    mass_mt: r.mass_mt,
                    inner_rad: r.inner_rad,
                    outer_rad: r.outer_rad,
                })
                .collect()
        })
        .unwrap_or_default();

    let materials = e
        .materials
        .as_ref()
        .map(|mv| {
            mv.iter()
                .map(|m| BodyMaterial {
                    name: m.name.clone(),
                    percent: m.percent,
                })
                .collect()
        })
        .unwrap_or_default();

    BodyScan {
        body_id: e.body_id,
        body_name: e.body_name.clone(),
        planet_class: e.planet_class.clone().unwrap_or_default(),
        landable: e.landable,
        scan_type: e.scan_type.clone().unwrap_or_default(),
        distance_from_arrival_ls: e.distance_from_arrival_ls.unwrap_or(0.0),
        radius: e.radius.unwrap_or(0.0),
        mass_em: e.mass_em.unwrap_or(0.0),
        surface_temperature: e.surface_temperature.unwrap_or(0.0),
        surface_gravity: e.surface_gravity.unwrap_or(0.0),
        tidal_lock: e.tidal_lock,
        volcanism: e.volcanism.clone().unwrap_or_default(),
        atmosphere: e.atmosphere.clone().unwrap_or_default(),
        terraform_state: e.terraform_state.clone().unwrap_or_default(),
        star_type: e.star_type.clone().unwrap_or_default(),
        parents,
        rings,
        materials,
        estimated_value: e.estimated_value.unwrap_or(0),
        composition: e.composition.as_ref().map(|c| BodyComposition {
            ice: c.ice,
            rock: c.rock,
            metal: c.metal,
        }),
    }
}

pub struct JournalReader {
    handle: Option<thread::JoinHandle<()>>,
    should_stop: std::sync::Arc<AtomicBool>,
    receiver: mpsc::Receiver<JournalData>,
}

impl JournalReader {
    pub fn start(journal_dir: PathBuf) -> Self {
        info!("Initializing journal reader for: {}", journal_dir.display());
        let (tx, rx) = mpsc::channel();
        let should_stop = std::sync::Arc::new(AtomicBool::new(false));
        let stop_flag = should_stop.clone();

        let handle = thread::spawn(move || {
            let mut journal_data = JournalData::new();

            info!("Loading existing journal files from: {}", journal_dir.display());
            load_existing_files(&journal_dir, &mut journal_data);
            info!("Loaded {} bodies from existing files", journal_data.bodies.len());
            let _ = tx.send(journal_data.clone());

            watch_latest_file(&journal_dir, &mut journal_data, &tx, &stop_flag);
        });

        Self {
            handle: Some(handle),
            should_stop,
            receiver: rx,
        }
    }

    pub fn try_recv(&self) -> Option<JournalData> {
        self.receiver.try_recv().ok()
    }

    pub fn stop(&mut self) {
        info!("Stopping journal reader");
        self.should_stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn restart(&mut self, journal_dir: PathBuf) {
        self.stop();
        *self = Self::start(journal_dir);
    }
}

impl Drop for JournalReader {
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }
}

fn load_existing_files(dir: &Path, data: &mut JournalData) {
    if !dir.exists() || !dir.is_dir() {
        warn!("Journal directory does not exist: {}", dir.display());
        return;
    }
    if let Some(active_file) = find_latest_journal_file(dir) {
        info!("Loading journal file: {}", active_file.display());
        read_file_lines(&active_file, data);
    } else {
        warn!("No journal file found in: {}", dir.display());
    }
}

fn watch_latest_file(
    dir: &Path,
    data: &mut JournalData,
    tx: &mpsc::Sender<JournalData>,
    stop_flag: &AtomicBool,
) {
    let mut last_file: Option<PathBuf> = None;
    let mut last_position: u64 = 0;

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            return;
        }

        if let Some(active) = find_latest_journal_file(dir) {
            let file_changed = last_file.as_ref() != Some(&active);
            if file_changed {
                info!("Journal file changed to: {}", active.display());
                last_file = Some(active.clone());
                last_position = 0;
                data.clear();
                read_file_lines(&active, data);
                last_position = active.metadata().map(|m| m.len()).unwrap_or(0);
                let _ = tx.send(data.clone());
            } else if let Ok(metadata) = active.metadata() {
                let file_size = metadata.len();
                if file_size > last_position {
                    if let Ok(mut file) = OpenOptions::new().read(true).open(&active) {
                        let _ = file.seek(SeekFrom::Start(last_position));
                        let reader = BufReader::new(file);
                        let mut changed = false;
                        for line in reader.lines().flatten() {
                            let trimmed = line.trim().to_owned();
                            if !trimmed.is_empty() {
                                data.process_line(&trimmed);
                                changed = true;
                            }
                        }
                        last_position = file_size;
                        if changed {
                            let _ = tx.send(data.clone());
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}

fn read_file_lines(path: &Path, data: &mut JournalData) {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                data.process_line(&line);
            }
        }
        Err(e) => error!("Failed to open journal file {}: {}", path.display(), e),
    }
}

fn find_latest_journal_file(dir: &Path) -> Option<PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            parse_journal_timestamp(&name).map(|ts| (e.path(), ts))
        })
        .collect();

    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.into_iter().next().map(|(path, _)| path)
}

fn parse_journal_timestamp(filename: &str) -> Option<u64> {
    // Journal.YYYY-MM-DDTHHMMSS.NN.log
    let parts: Vec<&str> = filename.split('.').collect();
    if parts.len() >= 3 && parts[0] == "Journal" && *parts.last()? == "log" {
        let d = parts[1]; // "YYYY-MM-DDTHHMMSS" (17 chars)
        if d.len() != 17 { return None; }
        let year: u64 = d[0..4].parse().ok()?;
        let month: u64 = d[5..7].parse().ok()?;
        let day: u64 = d[8..10].parse().ok()?;
        let hour: u64 = d[11..13].parse().ok()?;
        let min: u64 = d[13..15].parse().ok()?;
        let sec: u64 = d[15..17].parse().ok()?;
        Some(year * 10_000_000_000 + month * 100_000_000 + day * 1_000_000 + hour * 10_000 + min * 100 + sec)
    } else {
        None
    }
}

pub fn build_body_tree(bodies: &[BodyScan]) -> Vec<TreeNode> {
    use std::collections::{HashMap, HashSet};

    let mut body_map: HashMap<i32, &BodyScan> = HashMap::new();
    for body in bodies {
        body_map.insert(body.body_id, body);
    }

    let mut children_of: HashMap<i32, Vec<i32>> = HashMap::new();
    let mut has_known_parent: HashSet<i32> = HashSet::new();

    for (&body_id, body) in &body_map {
        if let Some(first_parent) = body.parents.first() {
            let parent_id = first_parent.body_id;
            if body_map.contains_key(&parent_id) {
                children_of.entry(parent_id).or_default().push(body_id);
                has_known_parent.insert(body_id);
            }
        }
    }

    let mut root_ids: Vec<i32> = body_map
        .keys()
        .filter(|&&id| !has_known_parent.contains(&id))
        .copied()
        .collect();
    root_ids.sort();

    root_ids
        .iter()
        .map(|&id| build_tree_node(id, &body_map, &children_of))
        .collect()
}

fn build_tree_node(
    body_id: i32,
    body_map: &std::collections::HashMap<i32, &BodyScan>,
    children_of: &std::collections::HashMap<i32, Vec<i32>>,
) -> TreeNode {
    let body = body_map[&body_id];
    let mut child_ids = children_of.get(&body_id).cloned().unwrap_or_default();
    child_ids.sort();
    TreeNode {
        name: body.body_name.clone(),
        body_id,
        children: child_ids
            .iter()
            .map(|&cid| build_tree_node(cid, body_map, children_of))
            .collect(),
        data: Some((*body).clone()),
    }
}

#[derive(Clone)]
pub struct TreeNode {
    pub name: String,
    pub body_id: i32,
    pub children: Vec<TreeNode>,
    pub data: Option<BodyScan>,
}
