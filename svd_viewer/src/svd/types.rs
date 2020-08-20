use svd_parser::{AddressBlock, Cpu, Field, Interrupt, RegisterProperties};

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub address_unit_bits: Option<u32>,
    pub width: Option<u32>,
    pub cpu: Option<Cpu>,
    pub peripherals: Vec<Peripheral>,
    pub default_register_properties: RegisterProperties,
}

impl From<svd_parser::Device> for Device {
    fn from(
        svd_parser::Device {
            name,
            version,
            description,
            address_unit_bits,
            width,
            cpu,
            peripherals,
            default_register_properties,
            ..
        }: svd_parser::Device,
    ) -> Self {
        Self {
            name,
            version,
            description,
            address_unit_bits,
            width,
            cpu,
            peripherals: peripherals.into_iter().map(From::from).collect(),
            default_register_properties,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Peripheral {
    pub name: String,
    pub version: Option<String>,
    pub display_name: Option<String>,
    pub group_name: Option<String>,
    pub description: Option<String>,
    pub base_address: u32,
    pub address_block: Option<AddressBlock>,
    pub interrupt: Vec<Interrupt>,
    pub default_register_properties: RegisterProperties,
    /// `None` indicates that the `<registers>` node is not present
    pub registers: Vec<Register>,
    pub derived_from: Option<String>,
}

impl From<svd_parser::Peripheral> for Peripheral {
    fn from(
        svd_parser::Peripheral {
            name,
            version,
            display_name,
            group_name,
            description,
            base_address,
            address_block,
            interrupt,
            default_register_properties,
            registers,
            derived_from,
            ..
        }: svd_parser::Peripheral,
    ) -> Self {
        Self {
            name,
            version,
            display_name,
            group_name,
            description,
            base_address,
            address_block,
            interrupt,
            default_register_properties,
            registers: registers
                .unwrap_or_else(|| vec![])
                .into_iter()
                .flat_map(|registers| register_cluster_to_registers(registers, base_address))
                .collect(),
            derived_from,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Register {
    pub name: String,
    pub description: Option<String>,
    pub address: u32,
    pub fields: Option<Vec<Field>>,
    pub value: u32,
}

fn register_info_to_register(
    svd_parser::RegisterInfo {
        name,
        description,
        address_offset,
        fields,
        ..
    }: svd_parser::RegisterInfo,
    base_address: u32,
) -> Register {
    Register {
        name: name,
        description,
        address: base_address + address_offset,
        fields,
        value: 0,
    }
}

fn register_cluster_to_registers(
    register_cluster: svd_parser::RegisterCluster,
    base_address: u32,
) -> Vec<Register> {
    match register_cluster {
        svd_parser::RegisterCluster::Register(register) => match register {
            svd_parser::Register::Single(single) => {
                vec![register_info_to_register(single, base_address)]
            }
            svd_parser::Register::Array(info, dim) => (0..dim.dim)
                .map(|d| {
                    let offset = dim.dim_increment * d;
                    let index = dim
                        .dim_index
                        .as_ref()
                        .map(|i| i.get(d as usize).map(Clone::clone))
                        .flatten()
                        .unwrap_or_else(|| d.to_string());

                    Register {
                        name: info.name.replace("%s", &index),
                        description: info.description.clone(),
                        address: base_address + offset + info.address_offset,
                        fields: info.fields.clone(),
                        value: 0,
                    }
                })
                .collect(),
        },
        svd_parser::RegisterCluster::Cluster(_cluster) => vec![],
    }
}
