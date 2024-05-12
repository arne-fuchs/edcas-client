use crate::edcas::{explorer::body::BodyType, EliteRustClient};
use crate::tui::{round_to_2, round_to_4, App};
use core::f64;
use ratatui::{prelude::*, style::Stylize, widgets::*};

pub fn tab_explorer(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &mut App,
) {
    // Data
    // Default data to display
    let mut data_system_info = vec![Row::new(vec!["no data".to_string()])];
    let mut data_system_name = Span::from("no data").light_red();
    let mut data_signals_list = vec![Row::new(vec!["no data".to_string()])];
    let mut data_body_list: Vec<Line> = vec![Line::styled("no data", Style::default().light_red())];
    //let mut data_body_signals_list = vec![Row::new(vec!["no data".to_string()])];
    let mut data_body_info: Vec<Row> =
        vec![Row::new(vec!["no data".to_string()]).style(Style::default().light_red())];
    let mut data_body_name = Span::from("no data").light_red();
    let mut data_system_gauge_scanned: i32 = 0;
    let mut data_system_gauge_all: i32 = 0;
    let mut data_system_gauge: f64 = 0.0 / 1.0;
    let mut data_planet_signals: Vec<Row> = vec![Row::new(vec!["no data".light_red()])]; //client.explorer.systems[index].planet_signals[index] (body_name, body_id, Vec signals)
    let mut additional_length_data_body_info: usize = 0;

    // Checks to not crash everything if list is empty
    // Data acquisition
    if !client.explorer.systems.is_empty() {
        data_system_name = Span::from(
            client.explorer.systems[client.explorer.index]
                .name
                .to_string(),
        );
        data_system_info = vec![
            //Row::new(vec!["Name".to_string(),client.explorer.systems[client.explorer.index].name.clone(),]),
            Row::new(vec![
                "Allegiance".to_string(),
                client.explorer.systems[client.explorer.index]
                    .allegiance
                    .clone(),
            ]),
            Row::new([
                "Economy".to_string(),
                client.explorer.systems[client.explorer.index]
                    .economy_localised
                    .clone(),
            ]),
            Row::new([
                "2. Economy".to_string(),
                client.explorer.systems[client.explorer.index]
                    .second_economy_localised
                    .clone(),
            ]),
            Row::new([
                "Government".to_string(),
                client.explorer.systems[client.explorer.index]
                    .government_localised
                    .clone(),
            ]),
            Row::new([
                "Security".to_string(),
                client.explorer.systems[client.explorer.index]
                    .security_localised
                    .clone(),
            ]),
            Row::new([
                "Population".to_string(),
                client.explorer.systems[client.explorer.index]
                    .population
                    .clone(),
            ]),
            Row::new([
                "Bodies".to_string(),
                client.explorer.systems[client.explorer.index]
                    .body_count
                    .clone(),
            ]),
            Row::new([
                "Non-bodies".to_string(),
                client.explorer.systems[client.explorer.index]
                    .non_body_count
                    .clone(),
            ]),
        ];

        data_signals_list = client.explorer.systems[client.explorer.index]
            .signal_list
            .iter()
            .map(|signal| Row::new(vec![signal.name.to_string(), signal.threat.to_string()]))
            .collect::<Vec<Row>>();

        if !client.explorer.systems[client.explorer.index]
            .planet_signals
            .is_empty()
        {
            data_planet_signals.clear();
            for planet_signal in &client.explorer.systems[client.explorer.index].planet_signals {
                for signal in &planet_signal.signals {
                    let signal_type: &str = if signal.type_localised != "null" {
                        signal.type_localised.as_str()
                    } else {
                        signal.r#type.as_str()
                    };

                    data_planet_signals.push(Row::new(vec![
                        planet_signal
                            .body_name
                            .trim_start_matches(
                                &client.explorer.systems[client.explorer.index].name,
                            )
                            .into(),
                        match signal_type {
                            "Biological" => signal_type.light_green(),
                            "Geological" => signal_type.magenta(),
                            _ => signal_type.into(),
                        },
                        signal.count.to_string().into(),
                    ]))
                }
            }
        } else {
            data_planet_signals.clear();
        }

        if client.explorer.systems[client.explorer.index].non_body_count != "N/A"
            && client.explorer.systems[client.explorer.index].body_count != "N/A"
        {
            data_system_gauge_scanned = client.explorer.systems[client.explorer.index]
                .body_list
                .len() as i32;

            data_system_gauge_all = client.explorer.systems[client.explorer.index]
                .non_body_count
                .parse::<i32>()
                .unwrap()
                + client.explorer.systems[client.explorer.index]
                    .body_count
                    .parse::<i32>()
                    .unwrap();

            if data_system_gauge_scanned > data_system_gauge_all {
                data_system_gauge_all = data_system_gauge_scanned; //shouldnt be the case
                                                                   //but it did crash one time i used
                                                                   //system signals as scanned
            }

            data_system_gauge = f64::from(data_system_gauge_scanned)
                / if data_system_gauge_all != 0 {
                    f64::from(data_system_gauge_all)
                } else {
                    1.0 //just to be sure
                };
        }

        if !client.explorer.systems[client.explorer.index]
            .body_list
            .is_empty()
        {
            //preparet to implement fancy tree, too dumb rn to do it.
            data_body_list.clear();
            for body in client.explorer.systems[client.explorer.index]
                .body_list
                .iter()
                .rev()
            {
                let mut space_string = "".to_string();

                for i in 0..body.get_parents().len() {
                    if i < body.get_parents().len() - 1 {
                        space_string.push_str("│  ")
                    } else {
                        space_string.push_str("│  "); //├
                    }
                }
                space_string.push_str(body.get_name().as_str());
                let signals_type_string: Vec<String> = body
                    .get_signals()
                    .iter()
                    .map(|body_signal| {
                        if body_signal.type_localised != "null" {
                            body_signal.type_localised.to_string()
                        } else {
                            body_signal.r#type.to_string()
                        }
                    })
                    .collect();
                data_body_list.push(
                    vec![
                        space_string.fg(Color::White),
                        " ".into(),
                        signals_type_string.join(" ").light_green().italic(),
                    ]
                    .into(),
                )
            }

            data_body_list.reverse();

            data_body_info = match &client.explorer.systems[client.explorer.index].body_list
                [client.explorer.systems[client.explorer.index].index]
            {
                BodyType::Star(star_body) => {
                    data_body_name = Span::from(star_body.body_name.to_string());
                    vec![
                        //Row::new(vec!["Timestamp".to_string(),star_body.timestamp.to_string(),]),
                        //Row::new(vec!["Event".to_string(), star_body.event.to_string()]),
                        //Row::new(vec!["Scan Type".to_string(),star_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), star_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), star_body.body_id.to_string()]),
                        //Row::new(vec!["System".to_string(),star_body.star_system.to_string(),]),
                        //Row::new(vec!["".to_string(), star_body.system_address.to_string()]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(star_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec!["Type".to_string(), star_body.star_type.to_string()]),
                        Row::new(vec!["Subclass".to_string(), star_body.subclass.to_string()]),
                        Row::new(vec![
                            "Solar Mass".to_string(),
                            round_to_4(star_body.stellar_mass).to_string(),
                        ]),
                        Row::new(vec![
                            "Solar Radius".to_string(),
                            round_to_4(star_body.radius / 695508000.0).to_string(),
                        ]),
                        Row::new(vec![
                            "Abs. Magnitude".to_string(),
                            star_body.absolute_magnitude.to_string(),
                        ]),
                        Row::new(vec![
                            "Age".to_string(),
                            [star_body.age_my.to_string(), "M.Years".to_string()].join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Temp".to_string(),
                            [star_body.surface_temperature.to_string(), "K".to_string()].join(" "),
                        ]),
                        Row::new(vec![
                            "Luminosity".to_string(),
                            star_body.luminosity.to_string(),
                        ]),
                        Row::new(vec![
                            "Semi major Axis".to_string(),
                            match star_body.semi_major_axis {
                                Some(sma) => [
                                    round_to_4(sma / 149597870700.0).to_string(),
                                    "AU".to_string(),
                                ]
                                .join(" "),
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Eccentricity".to_string(),
                            match star_body.eccentricity {
                                Some(sma) => round_to_4(sma).to_string(),
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Orb. Inclanation".to_string(),
                            match star_body.orbital_inclination {
                                Some(sma) => {
                                    [round_to_2(sma).to_string(), "°".to_string()].join("")
                                }
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Arg of Periapsis".to_string(),
                            match star_body.periapsis {
                                Some(sma) => {
                                    [round_to_2(sma).to_string(), "°".to_string()].join("")
                                }
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Orbital Period".to_string(),
                            match star_body.orbital_period {
                                Some(sma) => {
                                    [round_to_2(sma / 86400.0).to_string(), "D".to_string()]
                                        .join(" ")
                                }
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Mean Anomaly".to_string(),
                            match star_body.mean_anomaly {
                                Some(sma) => round_to_4(sma).to_string(),
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Rot. Period".to_string(),
                            [
                                round_to_2(star_body.rotation_period / 86400.0).to_string(),
                                "D".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Axial Tilt".to_string(),
                            [
                                round_to_2(star_body.axial_tilt * 180.0 / f64::consts::PI)
                                    .to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "was Discovered".to_string(),
                            if star_body.was_discovered {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "was Mapped".to_string(),
                            if star_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Asteroid Rings".to_string(),
                            star_body.asteroid_rings.len().to_string(),
                        ]),
                    ]
                }

                BodyType::BeltCluster(belt_body) => {
                    data_body_name = Span::from(belt_body.body_name.to_string());
                    vec![
                        //Row::new(vec!["Timestamp".to_string()belt_body.timestamp.to_string(),]),
                        //Row::new(vec!["Event".to_string(), belt_body.event.to_string()]),
                        //Row::new(vec!["Scan Type".to_string(),belt_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), belt_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), belt_body.body_id.to_string()]),
                        //Row::new(vec!["System".to_string(),belt_body.star_system.to_string(),]),
                        //Row::new(vec!["Parents".to_string(),belt_body.parents.len().to_string(),]),
                        //Row::new(vec!["System Address".to_string(), belt_body.system_address.to_string(),]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(belt_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Discovered".to_string(),
                            if belt_body.was_discovered {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Mapped".to_string(),
                            if belt_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                    ]
                }
                BodyType::Planet(planet_body) => {
                    data_body_name = Span::from(planet_body.body_name.to_string());
                    vec![
                        // Row::new(vec!["Timestamp".to_string(),planet_body.timestamp.to_string()]),
                        // Row::new(vec!["Event".to_string(), planet_body.event.to_string()]),
                        // Row::new(vec!"Scan Type".to_string()planet_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), planet_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), planet_body.body_id.to_string()]),
                        //Row::new(vec!["Parents".to_string(),planet_body.parents.len().to_string(),]),
                        //Row::new(vec!["System".to_string(),planet_body.star_system.to_string(),]),
                        //Row::new(vec!["".to_string(),planet_body.system_address.to_string()]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(planet_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        //Row::new(vec!["Tidal Lock".to_string(),planet_body.tidal_lock.to_string(),]),
                        Row::new(vec![
                            "Terraform State".to_string(),
                            planet_body.terraform_state.to_string(),
                        ]),
                        Row::new(vec![
                            "Class".to_string(),
                            planet_body.planet_class.to_string(),
                        ]),
                        Row::new(vec![
                            "Atmosphere".to_string(),
                            planet_body.atmosphere.to_string(),
                        ]),
                        Row::new(vec![
                            "Atmosphere Type".to_string(),
                            planet_body.atmosphere_type.to_string(),
                        ]),
                        Row::new(vec![
                            "Atmosphere Comp.".to_string(),
                            planet_body
                                .atmosphere_composition
                                .iter()
                                .map(|mat| {
                                    [
                                        mat.name.to_string(),
                                        " ".to_string(),
                                        mat.percent.to_string(),
                                        "%".to_string(),
                                    ]
                                    .join("")
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        ])
                        .height(
                            if planet_body.atmosphere_composition.is_empty() {
                                1
                            } else {
                                additional_length_data_body_info +=
                                    planet_body.atmosphere_composition.len() - 1;
                                planet_body.atmosphere_composition.len() as u16
                            },
                        ),
                        Row::new(vec![
                            "Volcanism".to_string(),
                            planet_body.volcanism.to_string(),
                        ]),
                        Row::new(vec![
                            "Earth Masses".to_string(),
                            round_to_4(planet_body.mass_em).to_string(),
                        ]),
                        Row::new(vec![
                            "Radius".to_string(),
                            [
                                round_to_2(planet_body.radius / 1000.0).to_string(),
                                "km".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Gravity".to_string(),
                            [
                                round_to_2(planet_body.surface_gravity * 0.1).to_string(),
                                "G".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Temp.".to_string(),
                            [
                                round_to_2(planet_body.surface_temperature).to_string(),
                                "K".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Pressure".to_string(),
                            [
                                round_to_4(planet_body.surface_pressure / 101325.0).to_string(),
                                //Pascal to atmosphere
                                "Atm".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Landable".to_string(),
                            if planet_body.landable {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Materials".to_string(),
                            planet_body
                                .materials
                                .iter()
                                .map(|mat| {
                                    [
                                        mat.name.to_string(),
                                        " ".to_string(),
                                        mat.percentage.to_string(),
                                        "%".to_string(),
                                    ]
                                    .join("")
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        ]),
                        Row::new(vec![
                            "Composition".to_string(),
                            planet_body
                                .composition
                                .iter()
                                .map(|mat| {
                                    [
                                        mat.name.to_string(),
                                        " ".to_string(),
                                        mat.percentage.to_string(),
                                        "%".to_string(),
                                    ]
                                    .join("")
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        ])
                        .height(if planet_body.composition.is_empty() {
                            1
                        } else {
                            additional_length_data_body_info += planet_body.composition.len() - 1;
                            planet_body.composition.len() as u16
                        }),
                        Row::new(vec![
                            "Semi major Axis".to_string(),
                            [
                                round_to_2(planet_body.semi_major_axis / 149597870700.0)
                                    .to_string(),
                                "AU".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Eccentricity".to_string(),
                            round_to_4(planet_body.eccentricity).to_string(),
                        ]),
                        Row::new(vec![
                            "Orb. Inclanation".to_string(),
                            [
                                round_to_2(planet_body.orbital_inclination).to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "Arg of Periapsis".to_string(),
                            [
                                round_to_2(planet_body.periapsis).to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "Orbital Period".to_string(),
                            [
                                round_to_2(planet_body.orbital_period / 86400.0).to_string(),
                                "D".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Ascending Node".to_string(),
                            round_to_2(planet_body.ascending_node).to_string(),
                        ]),
                        Row::new(vec![
                            "Mean Anomaly".to_string(),
                            round_to_2(planet_body.mean_anomaly).to_string(),
                        ]),
                        Row::new(vec![
                            "Rot. Period".to_string(),
                            [
                                round_to_2(planet_body.rotation_period / 86400.0).to_string(),
                                if planet_body.tidal_lock {
                                    "D [Tidally Locked]".to_string()
                                } else {
                                    "D".to_string()
                                },
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Axial Tilt".to_string(),
                            [
                                round_to_2(planet_body.axial_tilt * 180.0 / f64::consts::PI)
                                    .to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "Discovered".to_string(),
                            if planet_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Mapped".to_string(),
                            if planet_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Reserve Level".to_string(),
                            planet_body.reserve_level.to_string(),
                        ]),
                        Row::new(vec![
                            "Rings".to_string(),
                            planet_body.asteroid_rings.len().to_string(),
                        ]),
                        //Row::new(vec!["Signals".to_string(),planet_body.planet_signals.len().to_string(),]),
                    ]
                }
                BodyType::Ring(ring_body) => {
                    data_body_name = Span::from(ring_body.body_name.to_string());
                    vec![
                        //Row::new(vec!["Timestamp".to_string(),ring_body.timestamp.to_string(),]),
                        //Row::new(vec!["Event".to_string(), ring_body.event.to_string()]),
                        //Row::new(vec!["Scan Type".to_string(),ring_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), ring_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), ring_body.body_id.to_string()]),
                        //Row::new(vec!["Parents".to_string(),ring_body.parents.len().to_string(),]),
                        //Row::new(vec!["System".to_string(),ring_body.star_system.to_string(),]),
                        //Row::new(vec!["".to_string(), ring_body.system_address.to_string()]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(ring_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Semi major Axis".to_string(),
                            [
                                round_to_4(ring_body.semi_major_axis / 149597870700.0).to_string(),
                                "AU".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Arg of Periapsis".to_string(),
                            [round_to_2(ring_body.periapsis).to_string(), "°".to_string()].join(""),
                        ]),
                        Row::new(vec![
                            "Ascending Node".to_string(),
                            round_to_2(ring_body.ascending_node).to_string(),
                        ]),
                        Row::new(vec![
                            "Mean Anomaly".to_string(),
                            round_to_2(ring_body.mean_anomaly).to_string(),
                        ]),
                        Row::new(vec![
                            "Discovered".to_string(),
                            if ring_body.was_discovered {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Mapped".to_string(),
                            if ring_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        /*
                        Row::new(vec!["Signals".to_string(),ring_body.ring_signals.iter(.map(|sig| {[{
                                            if sig.type_localised != "null" {
                                                sig.type_localised.to_string()
                                            } else {
                                                sig.r#type.to_string()
                                            }},
                                        sig.count.to_string(),].join(" ")}).collect::<Vec<String>>().join("\n"),])
                        .height(if ring_body.ring_signals.is_empty() {1} else {
                            additional_length_data_body_info += ring_body.ring_signals.len() - 1;
                            ring_body.ring_signals.len() as u16
                        }),
                        */
                    ]
                }
            };

            // Selection from body_list (cursor and scrolling)
            app.body_list_state
                .select(Some(client.explorer.systems[client.explorer.index].index));
            /*
            if !client.explorer.systems[client.explorer.index].body_list
                [client.explorer.systems[client.explorer.index].index]
                .get_signals()
                .is_empty()
            {
                data_body_signals_list = client.explorer.systems[client.explorer.index].body_list
                    [client.explorer.systems[client.explorer.index].index]
                    .get_signals()
                    .iter()
                    .map(|body_signal| {
                        Row::new(vec![
                            if body_signal.type_localised != "null" {
                                body_signal.type_localised.clone()
                            } else {
                                body_signal.r#type.clone()
                            },
                            body_signal.count.to_string(),
                        ])
                    })
                    .collect::<Vec<Row>>();
            } else {
                data_body_signals_list =
                    vec![Row::new(vec!["no signals".to_string(), "".to_string()])];
            }*/
        }
    }

    // Layout definitions
    // general layout
    let layout_explorer = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Horizontal)
        .constraints(vec![
            Constraint::Fill(2),
            Constraint::Fill(3),
            Constraint::Fill(3),
        ])
        .split(chunk);

    // layout of "systems" Panel
    let layout_system = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Fill(1)])
        .split(layout_explorer[0]);

    // layout of "body information" panel
    let layout_body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(
                data_body_info.len() as u16 + additional_length_data_body_info as u16 + 1,
            ),
            Constraint::Fill(1), //Min(20),
        ])
        .split(layout_explorer[2]);

    // layout of system inforamtion
    let layout_system_info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(layout_system[0]);

    // Widget definitions
    let widget_system_info = Table::new(
        data_system_info,
        [Constraint::Length(10), Constraint::Fill(1)],
    )
    .block(
        Block::default()
            .title(Line::from(vec![
                Span::from(" < "),
                data_system_name,
                Span::from(" > "),
            ]))
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );

    let widget_system_gauge = LineGauge::default()
        .block(Block::default().borders(Borders::LEFT))
        .line_set(symbols::line::THICK)
        .gauge_style(Style::default().fg(Color::LightRed).bg(Color::Black))
        .label(format!(
            "{data_system_gauge_scanned}/{data_system_gauge_all}"
        ))
        .ratio(data_system_gauge);

    let widget_signal_list = Table::new(
        data_signals_list,
        [Constraint::Fill(1), Constraint::Length(2)],
    )
    .header(Row::new(vec!["Name", "TL"]))
    .block(
        Block::default()
            .title(" Signals ")
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );

    let widget_body_list = List::new(data_body_list) //List::new(data_body_list)
        .block(
            Block::default()
                .title(" Body List ")
                .borders(Borders::LEFT | Borders::TOP)
                .bold()
                .white(),
        )
        .highlight_style(Style::default().bold().on_dark_gray());

    let widget_body_info = Table::new(
        data_body_info,
        vec![Constraint::Length(16), Constraint::Fill(1)],
    )
    .block(
        Block::default()
            .title(Line::from(vec![
                Span::from(" Body: "),
                data_body_name,
                Span::from(" "),
            ]))
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );
    /*
    let widget_body_signals_list = Table::new(
        data_body_signals_list,
        [Constraint::Fill(1), Constraint::Length(6)],
    )
    .header(Row::new(vec!["Name", "Count"]))
    .block(
        Block::default()
            .title(" Body Signals ")
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );*/

    // TODO: make table scrollable how?
    let widget_planet_signals_list = Table::new(
        data_planet_signals,
        [
            Constraint::Length(8),
            Constraint::Fill(1),
            Constraint::Length(3),
        ],
    )
    .header(Row::new(vec!["Body", "Signal", "#"]))
    .block(
        Block::default()
            .title(" Body Signals ")
            .borders(Borders::TOP | Borders::LEFT)
            .bold(),
    );

    // render calls
    f.render_widget(widget_system_info, layout_system_info[0]);
    f.render_widget(widget_system_gauge, layout_system_info[1]);
    f.render_widget(widget_signal_list, layout_system[1]);

    f.render_stateful_widget(
        widget_body_list,
        layout_explorer[1],
        &mut app.body_list_state,
    );

    f.render_widget(widget_body_info, layout_body[0]);
    f.render_widget(widget_planet_signals_list, layout_body[1])
    //f.render_widget(widget_body_signals_list, layout_body[1]);
}
