# Ford Fusion 2017 - Complete Diagnostic Data Reference

This document provides a comprehensive overview of all configurable features and hidden diagnostic information available through OBD-II scanners (like FORScan) for the 2017 Ford Fusion SEL.

## Table of Contents
1. [Configurable Vehicle Features](#configurable-vehicle-features)
2. [Hidden Diagnostic Information](#hidden-diagnostic-information)
3. [CAN Network Architecture](#can-network-architecture)

---

## Configurable Vehicle Features

### HS-CAN Network Modules & Configurations

#### PCM (Powertrain Control Module) - 7E0
- Fuel type configuration (Regular/Premium/E85)
- Auto start-stop feature enable/disable
- Idle speed adjustment
- Engine performance parameters
- Emissions control settings
- Fuel economy optimization modes

#### IPC (Instrument Panel Cluster) - 720
**Display Settings:**
- Digital speedometer display
- Speed units (km/h or mph)
- Temperature units (°C or °F)
- Compass display enable/disable
- Welcome animation enable/disable
- Gauge brightness levels (8 settings)
- Ambient temperature display
- Auto-dim cluster at night

**Advanced Features:**
- Continuously Controlled Damping (CCD) suspension control
- Adaptive steering control
- Tire pressure monitoring activation
- Climate control integration
- Warning light configurations
- Trip computer settings

#### ABS Module
- AdvanceTrac settings
- Traction control sensitivity
- Electronic stability control parameters
- Brake assist configurations

#### RCM (Restraint Control Module)
- Airbag sensitivity settings
- Seatbelt warning configurations
- Occupant classification parameters

#### PSCM/SECM (Power Steering Modules)
- Steering assist levels
- Adaptive steering response
- Parking assist sensitivity

#### CCM (Cruise Control Module)
- Adaptive cruise control settings
- Speed limiter configurations
- Following distance parameters

#### PAM (Parking Aid Module)
- Parking sensor sensitivity
- Alert tone configurations
- Visual display settings

### MS-CAN Network Modules & Configurations

#### BCM/SJB (Body Control Module) - 726
**Lighting Systems:**
- Daytime Running Lights (DRL) enable/disable
- Automatic headlight control
- Welcome lighting duration (0-7 seconds)
- Fog light configurations
- Parking light behaviors
- Interior lighting settings

**Security & Convenience:**
- Auto lock when driving
- Unlock beep configurations (none/single/double/triple)
- Remote start enable/disable
- Keypad programming and enable/disable
- Memory settings for seats/mirrors
- Power point timeout settings
- Double honk functionality

**Electrical Systems:**
- Window auto-up/down settings
- Mirror configurations
- Wiper settings and rain sensing
- Various electrical load management

#### APIM (SYNC System) - 7D0
- Climate control integration in SYNC
- Navigation while driving settings
- Display shutdown behaviors
- Temperature display in status bar
- Media and connectivity preferences
- Voice command sensitivity

#### HVAC Module
- Climate control automatic settings
- Dual-zone temperature control
- Auto-start climate preferences
- Defrost configurations
- Air circulation settings

#### Audio Modules (ACM/DSP)
- Audio system configurations
- Speaker balance and fade
- Sound processing settings
- Satellite radio configurations

#### Seat Modules (DSM/DCSM)
- Heated/cooled seat settings
- Memory seat positions
- Massage seat configurations
- Lumbar support settings

#### Door Modules (DDM/PDM)
- Window auto-up/down configurations
- Door lock/unlock behaviors
- Approach lighting settings
- Mirror fold/unfold settings

#### Additional Advanced Modules
- **VDM**: Vehicle dynamics configurations
- **HUD**: Head-up display settings
- **SOD-L/R**: Side obstacle detection sensitivity
- **RFA**: Remote function programming
- **GSM**: Gear shift module preferences

---

## Hidden Diagnostic Information

### Engine & Powertrain Hidden Data

#### PCM (Powertrain Control Module)
- **Fuel trim values** (short-term and long-term)
- **Ignition timing advance/retard** per cylinder
- **Individual cylinder misfires** and combustion quality
- **Catalyst efficiency** and oxygen sensor response times
- **Evaporative emissions** system pressure and leak detection
- **Turbocharger boost pressure** and wastegate position
- **Variable valve timing** actuator positions
- **Direct injection pressure** and fuel rail pressure
- **Engine knock sensor** activity and knock retard
- **Throttle body** actual vs. commanded position
- **Mass airflow** vs. calculated airflow discrepancies
- **Exhaust gas recirculation (EGR)** valve position and flow
- **Engine oil life** percentage and degradation factors

#### Transmission Data
- **Gear ratios** and actual vs. commanded gear
- **Transmission fluid temperature** and pressure
- **Clutch slip** percentages and engagement timing
- **Shift solenoid** individual status and current draw
- **Torque converter** lockup status and slip
- **Line pressure** modulation and adaptive learning

### Body Systems Hidden Data

#### BCM (Body Control Module)
- **Battery voltage** under various load conditions
- **Charging system** performance and alternator output
- **Individual door lock** actuator feedback
- **Window motor** current draw and position feedback
- **Lighting circuits** individual bulb status and current draw
- **Wiper motor** position and load feedback
- **HVAC blower** actual vs. commanded speed
- **Seat heater** element resistance and current draw

#### Security & Access Systems
- **Key fob battery** voltage and signal strength
- **Passive entry** sensor status and range
- **Immobilizer** communication and authentication status
- **Remote start** system conditions and lockouts
- **Panic alarm** activation history and sensor status

### Safety Systems Hidden Data

#### ABS/ESC Module
- **Wheel speed sensors** individual readings and variance
- **Brake pressure** at each wheel during ABS events
- **Yaw rate sensor** and lateral acceleration data
- **Steering angle** sensor calibration and drift
- **Electronic stability** intervention frequency and severity
- **Brake fluid level** and brake pad wear indicators
- **Hill start assist** activation conditions and duration

#### Airbag System (RCM)
- **Crash sensor** sensitivity and calibration status
- **Seat occupancy** weight and position detection
- **Seatbelt** buckle status and pretensioner readiness
- **Side impact** sensor readings and thresholds
- **Airbag squib** resistance and continuity checks

### Comfort & Convenience Hidden Data

#### Climate Control (HVAC)
- **Cabin temperature** sensors (multiple zones)
- **Ambient air temperature** and humidity sensors
- **Refrigerant pressure** and AC compressor load
- **Blend door** actual positions vs. commanded
- **Air quality sensors** and cabin filter status
- **Automatic climate** learning algorithms and preferences

#### Instrument Panel Cluster (IPC)
- **Fuel level sender** resistance and calibration curves
- **Oil pressure** actual sensor voltage vs. displayed value
- **Coolant temperature** raw sensor data vs. gauge position
- **Speedometer** calibration factors and tire size compensation
- **Odometer** tamper detection and validation
- **Warning light** bulb-out detection and circuit integrity

### Advanced Driver Assistance Hidden Data

#### Parking Aid (PAM)
- **Ultrasonic sensor** individual range and sensitivity
- **Backup camera** image quality and lens cleanliness detection
- **Parking trajectory** calculation and steering input
- **Object detection** confidence levels and false positive rates

#### Adaptive Features
- **Adaptive cruise** radar sensor status and target tracking
- **Lane departure** camera calibration and line confidence
- **Blind spot** sensor range and detection algorithms
- **Cross-traffic** alert sensor coverage and object classification

### Communication & Infotainment Hidden Data

#### SYNC/APIM Module
- **GPS antenna** signal strength and satellite count
- **Bluetooth** pairing history and connection quality
- **USB port** power output and data transfer rates
- **Cellular modem** signal strength and data usage
- **Wi-Fi hotspot** connected devices and bandwidth usage
- **Software versions** and update history for all modules

### Vehicle Usage & Performance Analytics

#### Comprehensive Vehicle History
- **Engine operating hours** at various RPM ranges
- **Fuel consumption** patterns and efficiency trends
- **Brake application** frequency and intensity
- **Steering input** patterns and driving style analysis
- **Acceleration/deceleration** patterns and G-force data
- **Trip data** including routes, speeds, and idle times
- **Cold start** frequency and warm-up patterns
- **High-load events** and performance envelope usage

### Diagnostic & Maintenance Hidden Data

#### Module Health Monitoring
- **CAN bus** communication errors and message counts
- **Power supply** voltage fluctuations to individual modules
- **Ground circuit** integrity and resistance measurements
- **Module temperature** and thermal protection status
- **Software corruption** detection and checksum validation
- **Memory usage** and available storage in modules
- **Calibration drift** and sensor degradation tracking

#### Predictive Maintenance Data
- **Component lifecycle** tracking and replacement predictions
- **Fluid degradation** analysis and change intervals
- **Wear pattern** analysis for brakes, tires, and drivetrain
- **Environmental exposure** data (temperature extremes, vibration)
- **Usage severity** classification for warranty and service

---

## CAN Network Architecture

### HS-CAN Network Modules (500 Kbps)
- **PCM** (Powertrain Control Module)
- **ABS** (Anti-lock Brake System, includes AdvanceTrac and Traction Control)
- **RCM** (Restraint Control Module, AKA airbags and seatbelts)
- **AWD** (All Wheel Drive module, if equipped)
- **OCSM** (Occupant Classification System Module, AKA Passenger Seat Sensors)
- **PAM** (Parking Aid Module)
- **IPC** (Instrument Panel Cluster)
- **PSCM** (Power Steering Control Module, Hydraulic PS only)
- **SECM** (Steering Effort Control Module, Electric PS only)
- **CCM** (Cruise Control Module)
- **APIM** (Accessory Protocol Interface Module AKA SYNC)

### MS-CAN Network Modules (125 Kbps)
- **SJB/BCM** (Smart Junction Box/Body Control Module)
- **HVAC** (Heating, Ventilation & Air Conditioning module)
- **ACM** (Audio Control Module)
- **DSP** (Audio Digital Signal Processing Module)
- **DSM** (Driver Seat Module)
- **DDM** (Driver Door Module)
- **RFA** (Remote Function Actuator Module)
- **DCSM** (Dual Climate Controlled Seat Module)
- **SDARS** (Satellite Digital Audio Radio Service)
- **FCIM** (Front Controls Interface Module)
- **FDIM** (Front Display Interface Module)

### Additional Key Modules
- **VDM** (Vehicle Dynamics Module)
- **BECMB** (Battery Energy Control Module B)
- **SCCM** (Steering Column Control Module)
- **GSM** (Gear Shift Module)
- **TRCM** (Transmission Range Control Module)
- **PDM** (Passenger Door Module)
- **SOD-L** (Side Obstacle Detection Control Module LH)
- **SOD-R** (Side Obstacle Detection Control Module RH)
- **HUD** (Head Up Display Module)
- **DC/DC** Converter Control Module

### Network Structure
- **HS-CAN1 & HS-CAN2**: Direct diagnostic communication (500 Kbps)
- **HS-CAN3 & HS-CAN4**: Additional high-speed modules (500 Kbps)
- **MS-CAN**: Body control and comfort features (125 Kbps)
- **Gateway Module (GWM)**: Translates between all CAN networks and enables diagnostic tool access

---

## Important Notes

- **Safety First**: All configuration changes should be made with proper safety precautions
- **Backup**: Always create as-built backups before making any modifications
- **Professional Tools**: Use quality diagnostic tools like FORScan for configuration changes
- **Vehicle Specific**: This information is specific to 2017 Ford Fusion SEL models
- **Risk Warning**: Improper modifications can affect vehicle safety and operation

---

*This document serves as a reference for automotive diagnostic professionals and enthusiasts working with 2017 Ford Fusion vehicles.*