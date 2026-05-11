use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

type Entry = (&'static str, &'static str, &'static str);
type Section = (&'static str, &'static [Entry]);

const SECTIONS: &[Section] = &[
    (
        "Engine",
        &[
            (
                "RPM",
                "Revolutions Per Minute — crankshaft rotation speed",
                "Idle ~800 RPM, highway cruise ~2 200 RPM",
            ),
            (
                "MAF",
                "Mass Air Flow — air mass entering intake per second (g/s)",
                "8.2 g/s at idle",
            ),
            (
                "STFT B1/B2",
                "Short-Term Fuel Trim — real-time closed-loop fuel correction (%)",
                "+5% = running lean, ECU adds fuel",
            ),
            (
                "LTFT B1/B2",
                "Long-Term Fuel Trim — learned fuel bias stored in PCM memory (%)",
                "-3% = slight long-term richness",
            ),
            (
                "O2 B1S1/B1S2",
                "Oxygen sensor Bank 1 upstream (pre-cat) / downstream (post-cat)",
                "B1S1 oscillates 0.1-0.9 V at stoichiometry",
            ),
            (
                "EGR",
                "Exhaust Gas Recirculation — routes exhaust back to intake to cut NOx",
                "15% commanded at light cruise",
            ),
            (
                "Timing Advance",
                "Degrees before TDC the spark fires; retarded under knock",
                "20° advance at highway cruise",
            ),
            (
                "Engine Load",
                "Percentage of peak torque currently produced",
                "30% at steady cruise",
            ),
            (
                "Throttle Pos",
                "Throttle plate opening angle (0% closed → 100% WOT)",
                "12% at light acceleration",
            ),
            (
                "Oil Life",
                "ECU-estimated remaining oil quality (100% = fresh)",
                "15% = oil change imminent",
            ),
            (
                "Misfire Ct",
                "Combustion failures counted per cylinder",
                "Cyl 2 count = 12 over trip",
            ),
            (
                "Fuel Rate",
                "Fuel volume burned per hour (L/h)",
                "8.5 L/h at highway cruise",
            ),
            (
                "Torque %",
                "Engine torque as fraction of calibrated peak",
                "Driver demand 40%, actual 38%",
            ),
        ],
    ),
    (
        "Transmission",
        &[
            (
                "TCM",
                "Transmission Control Module — automatic gearbox ECU",
                "TCM commands upshift at 2 800 RPM",
            ),
            (
                "TC Slip",
                "Torque Converter Slip — speed delta between engine and trans input (%)",
                "5% during lockup engagement",
            ),
            (
                "TCC",
                "Torque Converter Clutch — locks converter at cruise for efficiency",
                "TCC engaged = 0% slip",
            ),
            (
                "LPC",
                "Line Pressure Control — solenoid regulating shift firmness",
                "80% duty = firm WOT shifts",
            ),
            (
                "SS1-SS4",
                "Shift Solenoids — hydraulic valves that engage gear clutch packs",
                "SS1 ON, SS2 OFF = 3rd gear",
            ),
            (
                "Line Pressure",
                "Hydraulic pressure inside the gearbox (kPa / PSI)",
                "700 kPa at WOT",
            ),
            (
                "Fluid Temp",
                "Transmission fluid temperature",
                "Operating range 70-120°C; >140°C = overheating",
            ),
        ],
    ),
    (
        "Body",
        &[
            (
                "BCM",
                "Body Control Module — manages locks, windows, lighting, battery",
                "BCM reports door-ajar status to IPC",
            ),
            (
                "Alternator %",
                "Alternator duty cycle — fraction of time it actively charges",
                "90% = heavy electrical load",
            ),
            (
                "Door Ajar",
                "Binary latch-sensor state per door / hood / trunk",
                "Driver: OPEN = door not fully closed",
            ),
        ],
    ),
    (
        "Safety",
        &[
            (
                "ABS",
                "Anti-lock Braking System — prevents wheel lock under hard braking",
                "Activates when wheel speed drops to 0 while car moves",
            ),
            (
                "ESC",
                "Electronic Stability Control — selectively brakes to prevent skid",
                "Intervenes when yaw rate exceeds predicted value",
            ),
            (
                "RCM",
                "Restraints Control Module — airbag and seatbelt ECU",
                "RCM deploys airbags when crash sensors exceed threshold",
            ),
            (
                "TPMS",
                "Tire Pressure Monitoring System — wheel-mounted pressure sensors",
                "Front: 240 kPa / 35 PSI",
            ),
            (
                "Yaw Rate",
                "Angular velocity of car rotating about its vertical axis (°/s)",
                "Turning left = positive yaw rate",
            ),
            (
                "Lat Accel",
                "Lateral (sideways) acceleration in g",
                "0.3 g in a moderate corner",
            ),
            (
                "Squib",
                "Pyrotechnic initiator in airbag circuit; checked for continuity",
                "Squib fault = airbag may not deploy",
            ),
            (
                "FL/FR/RL/RR",
                "Front-Left / Front-Right / Rear-Left / Rear-Right wheel positions",
                "FL wheel speed: 85 km/h",
            ),
        ],
    ),
    (
        "ADAS",
        &[
            (
                "ADAS",
                "Advanced Driver Assistance Systems — umbrella for active safety features",
                "PAM is one ADAS component",
            ),
            (
                "PAM",
                "Parking Aid Module — processes ultrasonic proximity data",
                "PAM triggers alert at 30 cm from obstacle",
            ),
            (
                "Ultrasonic",
                "Sound-pulse ranging sensor (0-250 cm range)",
                "Sensor reads 95 cm to rear wall",
            ),
            (
                "FC-L / FC-R",
                "Front Center-Left / Front Center-Right bumper sensor positions",
                "FC-L at 45 cm = obstacle near front center",
            ),
            (
                "RC-L / RC-R",
                "Rear Center-Left / Rear Center-Right bumper sensor positions",
                "RC-R at 20 cm = very close on rear-right",
            ),
        ],
    ),
    (
        "Climate",
        &[
            (
                "HVAC",
                "Heating, Ventilation & Air Conditioning — full climate system",
                "HVAC ECU controls blend doors, compressor, blower",
            ),
            (
                "Blend Door",
                "Motorized flap mixing hot and cooled air streams",
                "50% = equal mix; 0% = full cold",
            ),
            (
                "Evap Temp",
                "A/C evaporator coil temperature",
                "3°C prevents icing; <0°C = freeze-protection active",
            ),
            (
                "Refrig Pres",
                "Refrigerant high-side pressure (kPa / PSI)",
                "1 400 kPa normal; >2 000 kPa = high-pressure cutout",
            ),
            (
                "Comp Load",
                "A/C compressor duty cycle (%)",
                "100% = compressor always on, max cooling",
            ),
            (
                "Blower RPM",
                "Cabin fan motor speed",
                "1 200 RPM ≈ mid-speed setting",
            ),
        ],
    ),
    (
        "Infotainment",
        &[
            (
                "APIM",
                "Accessory Protocol Interface Module — Ford SYNC 3 head-unit ECU",
                "APIM manages navigation, BT, cellular",
            ),
            (
                "GPS",
                "Global Positioning System — satellite-based positioning",
                "12 satellites locked, GPS Fix",
            ),
            (
                "DGPS",
                "Differential GPS — ground-station corrections, ~1 m accuracy",
                "DGPS fix vs ~5 m for standard GPS",
            ),
            (
                "RTK",
                "Real-Time Kinematic GPS — carrier-phase corrections, <0.1 m accuracy",
                "RTK fix in survey-grade receivers",
            ),
            (
                "RSSI",
                "Received Signal Strength Indicator — cellular signal power (dBm)",
                "-80 dBm = moderate signal",
            ),
            (
                "dBm",
                "Decibels relative to 1 mW — logarithmic power unit",
                "-70 dBm is 10x stronger than -80 dBm",
            ),
            (
                "BT",
                "Bluetooth — short-range wireless pairing protocol",
                "2 BT devices connected",
            ),
        ],
    ),
    (
        "Analytics",
        &[
            (
                "RPM Histogram",
                "Distribution of engine-speed samples across four bands",
                "60% in Cruise band = efficient driving",
            ),
            (
                "Brake Event",
                "Detected deceleration exceeding 8 km/h/s threshold",
                "12 events = aggressive braking style",
            ),
            (
                "Hard Accel",
                "Detected acceleration exceeding 2.7 km/h/s threshold",
                "3 events = mostly gentle driving",
            ),
            (
                "Fuel Economy",
                "Combined speed + fuel-rate efficiency metric",
                "12 km/L / 8.3 L/100 km / 28.2 MPG",
            ),
            (
                "L/100 km",
                "Liters per 100 km — European fuel efficiency standard",
                "5 L/100 km = very efficient",
            ),
            (
                "MPG",
                "Miles Per Gallon — imperial fuel efficiency",
                "47 MPG = very efficient",
            ),
        ],
    ),
    (
        "Health",
        &[
            (
                "PCM",
                "Powertrain Control Module — primary engine / drivetrain ECU",
                "PCM: OK (3 s ago)",
            ),
            (
                "TCM",
                "Transmission Control Module — automatic gearbox controller",
                "TCM: STALE (45 s since last response)",
            ),
            (
                "IPC",
                "Instrument Panel Cluster — dashboard display ECU",
                "IPC: OK",
            ),
            (
                "DID",
                "Data Identifier — UDS protocol address for reading a value",
                "Heartbeat DID 0xF101",
            ),
            (
                "UDS",
                "Unified Diagnostic Services (ISO 14229) — ECU comms protocol",
                "Service 0x22 ReadDataByIdentifier",
            ),
            (
                "Heartbeat",
                "Periodic DID poll confirming a module is alive",
                "0xF101 success = module responding",
            ),
            (
                "UNKNOWN",
                "Module never contacted in this session",
                "Grey row = no attempt made yet",
            ),
            (
                "STALE",
                "Module last seen more than 30 s ago",
                "Yellow = likely sleeping or slow to respond",
            ),
            (
                "TIMEOUT",
                "3 or more consecutive read failures",
                "Red = module not responding",
            ),
        ],
    ),
];

pub struct GlossaryPanel {
    scroll_offset: usize,
}

impl GlossaryPanel {
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max = Self::total_lines().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + 1).min(max);
    }

    fn total_lines() -> usize {
        SECTIONS
            .iter()
            .fold(1, |acc, (_, entries)| acc + 1 + 1 + entries.len() * 2 + 1)
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let mut lines: Vec<Spans> = vec![Spans::from("")];

        for (section, entries) in SECTIONS {
            lines.push(Spans::from(Span::styled(
                format!("  {}", section),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Spans::from(""));

            for (term, def, example) in *entries {
                lines.push(Spans::from(vec![
                    Span::styled(
                        format!("  {:<16}", term),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(*def, Style::default().fg(Color::Gray)),
                ]));
                lines.push(Spans::from(Span::styled(
                    format!("  {:<16}  ex: {}", "", example),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            lines.push(Spans::from(""));
        }

        let para = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Glossary"))
            .scroll((self.scroll_offset as u16, 0));
        f.render_widget(para, area);
    }

    pub fn help_text(&self) -> &str {
        "up/down or j/k: scroll glossary"
    }
}
