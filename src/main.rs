use std::{net::IpAddr, str::FromStr};

use fronius::{DeviceId, Fronius};
use influxdb2::Client;
use influxdb2_derive::WriteDataPoint;
use chrono::prelude::*;
mod fronius;

#[derive(Debug)]
struct OptionEmptyError {
    variable_name: String
}

impl std::error::Error for OptionEmptyError {}

impl std::fmt::Display for OptionEmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Could not access option value for '{}'", self.variable_name)
    }
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "inverter"]
struct InverterData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    ac_power: Option<f64>,
    #[influxdb(field)]
    ac_power_abs: Option<f64>,
    #[influxdb(field)]
    ac_current: Option<f64>,
    #[influxdb(field)]
    ac_voltage: Option<f64>,
    #[influxdb(field)]
    ac_frequency: Option<f64>,
    #[influxdb(field)]
    dc_current: Option<f64>,
    #[influxdb(field)]
    dc_voltage: Option<f64>,
    #[influxdb(field)]
    total_energy: Option<f64>,
    #[influxdb(timestamp)]
    time: i64,
}


fn get_inverter_data(fronius: &Fronius, device_id: &DeviceId) -> Result<InverterData, Box<dyn std::error::Error>> {
    let response = fronius.get_inverter_realtime_data_device::<fronius::CommonInverterData>(device_id.to_owned())?;

    let data = InverterData {
        device: "Inverter".to_owned(),
        ac_power: response.pac.value,
        ac_power_abs: response.sac.value,
        ac_current: response.iac.value,
        ac_voltage: response.uac.value,
        ac_frequency: match response.fac {
            None => None,
            Some(a) => a.value,
        } ,
        dc_current: response.idc.value,
        dc_voltage: response.udc.value,
        total_energy: response.total_energy.value,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "inverter_phase"]
struct InverterPhaseData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    ac_l1_current: Option<f64>,
    #[influxdb(field)]
    ac_l2_current: Option<f64>,
    #[influxdb(field)]
    ac_l3_current: Option<f64>,
    #[influxdb(field)]
    dc_l1_voltage: Option<f64>,
    #[influxdb(field)]
    dc_l2_voltage: Option<f64>,
    #[influxdb(field)]
    dc_l3_voltage: Option<f64>,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_inverter_phase_data(fronius: &Fronius, device_id: &DeviceId) -> Result<InverterPhaseData, Box<dyn std::error::Error>> {
    let response = fronius.get_inverter_realtime_data_device::<fronius::ThreePhaseInverterData>(device_id.to_owned())?;
    let data = InverterPhaseData {
        device: "Inverter".to_owned(),
        ac_l1_current: response.iac_l1.value,
        ac_l2_current: response.iac_l2.value,
        ac_l3_current: response.iac_l3.value,
        dc_l1_voltage: response.uac_l1.value,
        dc_l2_voltage: response.uac_l2.value,
        dc_l3_voltage: response.uac_l3.value,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "inverter_info"]
struct InverterInfo {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    device_type: i64,
    #[influxdb(field)]
    pv_power: i64,
    #[influxdb(field)]
    name: String,
    #[influxdb(field)]
    is_visualized: bool,
    #[influxdb(field)]
    id: String,
    #[influxdb(field)]
    error_code: i64,
    #[influxdb(field)]
    status_code: String,
    #[influxdb(field)]
    state: String,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_inverter_info(fronius: &Fronius, device_id: &DeviceId) -> Result<InverterInfo, Box<dyn std::error::Error>> {
    let device_id = u8::from(device_id.to_owned()).to_string();
    let res = fronius.get_inverter_info()?;
    let response = res[&device_id].as_ref().expect("Invalid device id");
    let data = InverterInfo {
        device: "Inverter".to_owned(),
        device_type: response.dt,
        pv_power: response.pv_power,
        name: response.custom_name.to_owned(),
        is_visualized: response.show > 0,
        id: response.unique_id.to_owned(),
        error_code: response.error_code,
        status_code: response.status_code.to_string(),
        state: response.inverter_state.to_owned(),
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "meter"]
struct MeterData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    l1_current: Option<f64>,
    #[influxdb(field)]
    l2_current: Option<f64>,
    #[influxdb(field)]
    l3_current: Option<f64>,
    #[influxdb(field)]
    current: Option<f64>,
    #[influxdb(field)]
    l1_voltage: Option<f64>,
    #[influxdb(field)]
    l2_voltage: Option<f64>,
    #[influxdb(field)]
    l3_voltage: Option<f64>,
    #[influxdb(field)]
    l12_voltage: Option<f64>,
    #[influxdb(field)]
    l23_voltage: Option<f64>,
    #[influxdb(field)]
    l31_voltage: Option<f64>,
    #[influxdb(field)]
    l1_power: Option<f64>,
    #[influxdb(field)]
    l2_power: Option<f64>,
    #[influxdb(field)]
    l3_power: Option<f64>,
    #[influxdb(field)]
    power: f64,
    #[influxdb(field)]
    frequency_average: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_meter_data(fronius: &Fronius, device_id: &DeviceId) -> Result<MeterData, Box<dyn std::error::Error>> {
    let response = fronius.get_meter_realtime_data_device(device_id)?;
    let data = MeterData {
        device: "Meter".to_owned(),
        l1_current: response.current_ac_phase_1,
        l2_current: response.current_ac_phase_2,
        l3_current: response.current_ac_phase_3,
        current: response.current_ac_sum,
        l1_voltage: response.voltage_ac_phase_1,
        l2_voltage: response.voltage_ac_phase_2,
        l3_voltage: response.voltage_ac_phase_3,
        l12_voltage: response.voltage_ac_phase_to_phase_12,
        l23_voltage: response.voltage_ac_phase_to_phase_23,
        l31_voltage: response.voltage_ac_phase_to_phase_31,
        l1_power: response.power_real_p_phase_1,
        l2_power: response.power_real_p_phase_2,
        l3_power: response.power_real_p_phase_3,
        power: response.power_real_p_sum,
        frequency_average: response.frequency_phase_average,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "storage"]
struct StorageData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    enabled: bool,
    #[influxdb(field)]
    charge_percentage: f64,
    #[influxdb(field)]
    capacity: f64,
    #[influxdb(field)]
    dc_current: f64,
    #[influxdb(field)]
    dc_voltage: f64,
    #[influxdb(field)]
    temperature_cell: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_storage_data(fronius: &Fronius, device_id: &DeviceId) -> Result<StorageData, Box<dyn std::error::Error>> {
    let response = fronius.get_storage_realtime_data_device(device_id)?;
    let data = StorageData {
        device: "Storage".to_owned(),
        enabled: response.controller.enable > 0,
        charge_percentage: response.controller.state_of_charge_relative,
        capacity: response.controller.capacity_maximum,
        dc_current: response.controller.current_dc,
        dc_voltage: response.controller.voltage_dc,
        temperature_cell: response.controller.temperature_cell,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "ohm_pilot"]
struct OhmPilotData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    state: String,
    #[influxdb(field)]
    error_code: i64,
    #[influxdb(field)]
    power: f64,
    #[influxdb(field)]
    temperature: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn get_ohm_pilot_data(fronius: &Fronius, device_id: &DeviceId) -> Result<OhmPilotData, Box<dyn std::error::Error>> {
    let response = fronius.get_ohm_pilot_realtime_data_device(device_id)?;
    let data = OhmPilotData {
        device: "OhmPilot".to_owned(),
        state: response.code_of_state.to_string(),
        error_code: response.code_of_error.unwrap_or(0),
        power: response.power_real_pac_sum,
        temperature: response.temperature_channel_1,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

#[derive(Default, Debug, WriteDataPoint)]
#[measurement = "power_flow"]
struct PowerFlowData {
    #[influxdb(tag)]
    device: String,
    #[influxdb(field)]
    akku: Option<f64>,
    #[influxdb(field)]
    grid: Option<f64>,
    #[influxdb(field)]
    load: Option<f64>,
    #[influxdb(field)]
    photovoltaik: f64,
    #[influxdb(field)]
    relative_autonomy: Option<f64>,
    #[influxdb(field)]
    relative_self_consumption: Option<f64>,
    #[influxdb(timestamp)]
    time: i64
}


fn get_power_flow_data(fronius: &Fronius) -> Result<PowerFlowData, Box<dyn std::error::Error>> {
    let response = fronius.get_power_flow_realtime_data()?;
    let data = PowerFlowData {
        device: "Unknown".to_owned(),
        akku: response.site.p_akku,
        grid: response.site.p_grid,
        load: response.site.p_load,
        photovoltaik: response.site.p_pv,
        relative_autonomy: response.site.rel_autonomy,
        relative_self_consumption: response.site.rel_self_consumption,
        time: Utc::now().timestamp_nanos_opt().expect("Could not fetch timestamp"),
    };
    Ok(data)
}

fn send_to_influx(client: &Client, bucket: &str,  data: impl futures::Stream<Item = impl influxdb2::models::WriteDataPoint> + Send + Sync + 'static){
    let res = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.write(bucket, data));

    if let Err(error) = res {
        println!("Error during influxdb write occured: {:?}", error);
    }
}



fn fetch_data(fronius: &Fronius) -> Result<(), Box<dyn std::error::Error>> {
    let interver_id = DeviceId::try_from(1).unwrap();
    let meter_id = DeviceId::try_from(0).unwrap();
    let storage_id = DeviceId::try_from(0).unwrap();
    let ohm_pilot_id = DeviceId::try_from(0).unwrap();
    let inverter_data = get_inverter_data(fronius, &interver_id);
    let inverter_phase_data = get_inverter_phase_data(fronius, &interver_id);
    let inverter_info = get_inverter_info(fronius, &interver_id);
    let meter_data = get_meter_data(fronius, &meter_id);
    let storage_data = get_storage_data(fronius, &storage_id);
    let ohm_pilot_data = get_ohm_pilot_data(fronius, &ohm_pilot_id);
    let power_flow_data = get_power_flow_data(fronius);

    let client = Client::new(std::env::var("INFLUX_DB_URL")?, std::env::var("INFLUX_DB_ORG")?, std::env::var("INFLUX_DB_TOKEN")?);
    let bucket = std::env::var("INFLUX_DB_BUCKET")?;

    if let Ok(val) = inverter_data {
        send_to_influx(&client, &bucket, futures::stream::iter(vec![val]));
    }else if let Err(error) = inverter_data {
        println!("Error during fetch of inverter_data occured: {:?}", error);
    }

    if let Ok(val) = inverter_phase_data {
        send_to_influx(&client, &bucket, futures::stream::iter(vec![val]));
    }else if let Err(error) = inverter_phase_data {
        println!("Error during fetch of inverter_phase_data occured: {:?}", error);
    }

    if let Ok(val) = inverter_info {
        send_to_influx(&client, &bucket, futures::stream::iter(vec![val]));
    }else if let Err(error) = inverter_info {
        println!("Error during fetch of inverter_info occured: {:?}", error);
    }

    if let Ok(val) = meter_data {
        send_to_influx(&client, &bucket, futures::stream::iter(vec![val]));
    }else if let Err(error) = meter_data {
        println!("Error during fetch of meter_data occured: {:?}", error);
    }

    if let Ok(val) = storage_data {
        send_to_influx(&client, &bucket, futures::stream::iter(vec![val]));
    }else if let Err(error) = storage_data {
        println!("Error during fetch of storage_data occured: {:?}", error);
    }

    if let Ok(val) = ohm_pilot_data {
        send_to_influx(&client, &bucket, futures::stream::iter(vec![val]));
    }else if let Err(error) = ohm_pilot_data {
        println!("Error during fetch of ohm_pilot_data occured: {:?}", error);
    }

    if let Ok(val) = power_flow_data {
        send_to_influx(&client, &bucket, futures::stream::iter(vec![val]));
    }else if let Err(error) = power_flow_data {
        println!("Error during fetch of power_flow_data occured: {:?}", error);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip_str = std::env::var("FRONIUS_IP")?;
    let ip = IpAddr::V4(std::net::Ipv4Addr::from_str(&ip_str)?);
    let fronius = Fronius::connect(ip)?;
    loop {
        let now = Utc::now();
        println!("Reporting data at: {now}");
        let res = fetch_data(&fronius);

        if let Err(error) = res {
            println!("Error during fetch occured: {:?}", error);
        }else{
            res?;
        }
        std::thread::sleep(std::time::Duration::from_secs(15));
    }
}
