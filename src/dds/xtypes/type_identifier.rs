use speedy::{Context, Readable, Reader, Writable, Writer};

pub type EquivalenceHash = [u8; 14];

// DDS-XTypes v1.3 §7.3.1 TypeIdentifier discriminant values
pub const TK_NONE: u8 = 0x00;
pub const TK_BOOLEAN: u8 = 0x01;
pub const TK_BYTE: u8 = 0x02;
pub const TK_INT16: u8 = 0x03;
pub const TK_INT32: u8 = 0x04;
pub const TK_INT64: u8 = 0x05;
pub const TK_UINT16: u8 = 0x06;
pub const TK_UINT32: u8 = 0x07;
pub const TK_UINT64: u8 = 0x08;
pub const TK_FLOAT32: u8 = 0x09;
pub const TK_FLOAT64: u8 = 0x0A;
pub const TK_FLOAT128: u8 = 0x0B;
pub const TK_INT8: u8 = 0x0C;
pub const TK_UINT8: u8 = 0x0D;
pub const TK_CHAR8: u8 = 0x10;
pub const TK_CHAR16: u8 = 0x11;

pub const TI_STRING8_SMALL: u8 = 0x70;
pub const TI_STRING8_LARGE: u8 = 0x71;
pub const TI_STRING16_SMALL: u8 = 0x72;
pub const TI_STRING16_LARGE: u8 = 0x73;

pub const TI_PLAIN_SEQUENCE_SMALL: u8 = 0x80;
pub const TI_PLAIN_SEQUENCE_LARGE: u8 = 0x81;
pub const TI_PLAIN_ARRAY_SMALL: u8 = 0x90;
pub const TI_PLAIN_ARRAY_LARGE: u8 = 0x91;
pub const TI_PLAIN_MAP_SMALL: u8 = 0xA0;
pub const TI_PLAIN_MAP_LARGE: u8 = 0xA1;

pub const TI_STRONGLY_CONNECTED_COMPONENT: u8 = 0xB0;

pub const EK_COMPLETE: u8 = 0xF2;
pub const EK_MINIMAL: u8 = 0xF1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeIdentifier {
    Primitive(u8),
    String8Small { bound: u8 },
    String8Large { bound: u32 },
    String16Small { bound: u8 },
    String16Large { bound: u32 },
    PlainSequenceSmall {
        header: PlainCollectionHeader,
        bound: u8,
        element_identifier: Box<TypeIdentifier>,
    },
    PlainSequenceLarge {
        header: PlainCollectionHeader,
        bound: u32,
        element_identifier: Box<TypeIdentifier>,
    },
    PlainArraySmall {
        header: PlainCollectionHeader,
        bound: u8,
        element_identifier: Box<TypeIdentifier>,
    },
    PlainArrayLarge {
        header: PlainCollectionHeader,
        bound: u32,
        element_identifier: Box<TypeIdentifier>,
    },
    MinimalHash(EquivalenceHash),
    CompleteHash(EquivalenceHash),
    StronglyConnectedComponent(StronglyConnectedComponentId),
    Extended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainCollectionHeader {
    pub equiv_kind: u8,
    pub element_flags: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StronglyConnectedComponentId {
    pub sc_component_id: EquivalenceHash,
    pub scc_length: i32,
    pub scc_index: i32,
}

impl<'a, C: Context> Readable<'a, C> for PlainCollectionHeader {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let equiv_kind = reader.read_u8()?;
        let element_flags = reader.read_u16()?;
        Ok(Self {
            equiv_kind,
            element_flags,
        })
    }
}

impl<C: Context> Writable<C> for PlainCollectionHeader {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u8(self.equiv_kind)?;
        writer.write_u16(self.element_flags)?;
        Ok(())
    }
}

impl<'a, C: Context> Readable<'a, C> for StronglyConnectedComponentId {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let mut sc_component_id = [0u8; 14];
        reader.read_bytes(&mut sc_component_id)?;
        let scc_length = reader.read_i32()?;
        let scc_index = reader.read_i32()?;
        Ok(Self {
            sc_component_id,
            scc_length,
            scc_index,
        })
    }
}

impl<C: Context> Writable<C> for StronglyConnectedComponentId {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_slice(&self.sc_component_id)?;
        writer.write_i32(self.scc_length)?;
        writer.write_i32(self.scc_index)?;
        Ok(())
    }
}

fn is_primitive(disc: u8) -> bool {
    matches!(
        disc,
        TK_NONE
            | TK_BOOLEAN
            | TK_BYTE
            | TK_INT16
            | TK_INT32
            | TK_INT64
            | TK_UINT16
            | TK_UINT32
            | TK_UINT64
            | TK_FLOAT32
            | TK_FLOAT64
            | TK_FLOAT128
            | TK_INT8
            | TK_UINT8
            | TK_CHAR8
            | TK_CHAR16
    )
}

impl<'a, C: Context> Readable<'a, C> for TypeIdentifier {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let disc = reader.read_u8()?;

        if is_primitive(disc) {
            return Ok(TypeIdentifier::Primitive(disc));
        }

        match disc {
            TI_STRING8_SMALL => {
                let bound = reader.read_u8()?;
                Ok(TypeIdentifier::String8Small { bound })
            }
            TI_STRING8_LARGE => {
                let bound = reader.read_u32()?;
                Ok(TypeIdentifier::String8Large { bound })
            }
            TI_STRING16_SMALL => {
                let bound = reader.read_u8()?;
                Ok(TypeIdentifier::String16Small { bound })
            }
            TI_STRING16_LARGE => {
                let bound = reader.read_u32()?;
                Ok(TypeIdentifier::String16Large { bound })
            }
            TI_PLAIN_SEQUENCE_SMALL => {
                let header: PlainCollectionHeader = reader.read_value()?;
                let bound = reader.read_u8()?;
                let element_identifier: TypeIdentifier = reader.read_value()?;
                Ok(TypeIdentifier::PlainSequenceSmall {
                    header,
                    bound,
                    element_identifier: Box::new(element_identifier),
                })
            }
            TI_PLAIN_SEQUENCE_LARGE => {
                let header: PlainCollectionHeader = reader.read_value()?;
                let bound = reader.read_u32()?;
                let element_identifier: TypeIdentifier = reader.read_value()?;
                Ok(TypeIdentifier::PlainSequenceLarge {
                    header,
                    bound,
                    element_identifier: Box::new(element_identifier),
                })
            }
            TI_PLAIN_ARRAY_SMALL => {
                let header: PlainCollectionHeader = reader.read_value()?;
                let bound = reader.read_u8()?;
                let element_identifier: TypeIdentifier = reader.read_value()?;
                Ok(TypeIdentifier::PlainArraySmall {
                    header,
                    bound,
                    element_identifier: Box::new(element_identifier),
                })
            }
            TI_PLAIN_ARRAY_LARGE => {
                let header: PlainCollectionHeader = reader.read_value()?;
                let bound = reader.read_u32()?;
                let element_identifier: TypeIdentifier = reader.read_value()?;
                Ok(TypeIdentifier::PlainArrayLarge {
                    header,
                    bound,
                    element_identifier: Box::new(element_identifier),
                })
            }
            EK_MINIMAL => {
                let mut hash = [0u8; 14];
                reader.read_bytes(&mut hash)?;
                Ok(TypeIdentifier::MinimalHash(hash))
            }
            EK_COMPLETE => {
                let mut hash = [0u8; 14];
                reader.read_bytes(&mut hash)?;
                Ok(TypeIdentifier::CompleteHash(hash))
            }
            TI_STRONGLY_CONNECTED_COMPONENT => {
                let scc: StronglyConnectedComponentId = reader.read_value()?;
                Ok(TypeIdentifier::StronglyConnectedComponent(scc))
            }
            _ => Ok(TypeIdentifier::Extended),
        }
    }
}

impl<C: Context> Writable<C> for TypeIdentifier {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        match self {
            TypeIdentifier::Primitive(disc) => {
                writer.write_u8(*disc)?;
            }
            TypeIdentifier::String8Small { bound } => {
                writer.write_u8(TI_STRING8_SMALL)?;
                writer.write_u8(*bound)?;
            }
            TypeIdentifier::String8Large { bound } => {
                writer.write_u8(TI_STRING8_LARGE)?;
                writer.write_u32(*bound)?;
            }
            TypeIdentifier::String16Small { bound } => {
                writer.write_u8(TI_STRING16_SMALL)?;
                writer.write_u8(*bound)?;
            }
            TypeIdentifier::String16Large { bound } => {
                writer.write_u8(TI_STRING16_LARGE)?;
                writer.write_u32(*bound)?;
            }
            TypeIdentifier::PlainSequenceSmall {
                header,
                bound,
                element_identifier,
            } => {
                writer.write_u8(TI_PLAIN_SEQUENCE_SMALL)?;
                writer.write_value(header)?;
                writer.write_u8(*bound)?;
                writer.write_value(element_identifier.as_ref())?;
            }
            TypeIdentifier::PlainSequenceLarge {
                header,
                bound,
                element_identifier,
            } => {
                writer.write_u8(TI_PLAIN_SEQUENCE_LARGE)?;
                writer.write_value(header)?;
                writer.write_u32(*bound)?;
                writer.write_value(element_identifier.as_ref())?;
            }
            TypeIdentifier::PlainArraySmall {
                header,
                bound,
                element_identifier,
            } => {
                writer.write_u8(TI_PLAIN_ARRAY_SMALL)?;
                writer.write_value(header)?;
                writer.write_u8(*bound)?;
                writer.write_value(element_identifier.as_ref())?;
            }
            TypeIdentifier::PlainArrayLarge {
                header,
                bound,
                element_identifier,
            } => {
                writer.write_u8(TI_PLAIN_ARRAY_LARGE)?;
                writer.write_value(header)?;
                writer.write_u32(*bound)?;
                writer.write_value(element_identifier.as_ref())?;
            }
            TypeIdentifier::MinimalHash(hash) => {
                writer.write_u8(EK_MINIMAL)?;
                writer.write_slice(hash)?;
            }
            TypeIdentifier::CompleteHash(hash) => {
                writer.write_u8(EK_COMPLETE)?;
                writer.write_slice(hash)?;
            }
            TypeIdentifier::StronglyConnectedComponent(scc) => {
                writer.write_u8(TI_STRONGLY_CONNECTED_COMPONENT)?;
                writer.write_value(scc)?;
            }
            TypeIdentifier::Extended => {
                writer.write_u8(0xFE)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeIdentifierWithSize {
    pub type_id: TypeIdentifier,
    pub typeobject_serialized_size: u32,
}

impl<'a, C: Context> Readable<'a, C> for TypeIdentifierWithSize {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let _dheader = reader.read_u32()?;
        let type_id: TypeIdentifier = reader.read_value()?;
        let typeobject_serialized_size = reader.read_u32()?;
        Ok(Self {
            type_id,
            typeobject_serialized_size,
        })
    }
}

impl TypeIdentifier {
    fn serialized_size(&self) -> usize {
        match self {
            TypeIdentifier::Primitive(_) => 1,
            TypeIdentifier::String8Small { .. } | TypeIdentifier::String16Small { .. } => 2,
            TypeIdentifier::String8Large { .. } | TypeIdentifier::String16Large { .. } => 5,
            TypeIdentifier::MinimalHash(_) | TypeIdentifier::CompleteHash(_) => 15,
            TypeIdentifier::PlainSequenceSmall { element_identifier, .. } => {
                1 + 3 + 1 + element_identifier.serialized_size()
            }
            TypeIdentifier::PlainSequenceLarge { element_identifier, .. } => {
                1 + 3 + 4 + element_identifier.serialized_size()
            }
            TypeIdentifier::PlainArraySmall { element_identifier, .. } => {
                1 + 3 + 1 + element_identifier.serialized_size()
            }
            TypeIdentifier::PlainArrayLarge { element_identifier, .. } => {
                1 + 3 + 4 + element_identifier.serialized_size()
            }
            TypeIdentifier::StronglyConnectedComponent(_) => 1 + 14 + 4 + 4,
            TypeIdentifier::Extended => 1,
        }
    }
}

impl TypeIdentifierWithSize {
    fn content_size(&self) -> usize {
        self.type_id.serialized_size() + 4
    }
}

impl TypeIdentifierWithDependencies {
    fn content_size(&self) -> usize {
        let tws_size = 4 + self.typeid_with_size.content_size();
        let deps_size: usize = self
            .dependent_typeids
            .iter()
            .map(|d| 4 + d.content_size())
            .sum();
        tws_size + 4 + deps_size
    }
}

impl<C: Context> Writable<C> for TypeIdentifierWithSize {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u32(self.content_size() as u32)?;
        writer.write_value(&self.type_id)?;
        writer.write_u32(self.typeobject_serialized_size)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeIdentifierWithDependencies {
    pub typeid_with_size: TypeIdentifierWithSize,
    pub dependent_typeids: Vec<TypeIdentifierWithSize>,
}

impl<'a, C: Context> Readable<'a, C> for TypeIdentifierWithDependencies {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let _dheader = reader.read_u32()?;
        let typeid_with_size: TypeIdentifierWithSize = reader.read_value()?;

        let count = reader.read_i32()?;
        let mut dependent_typeids = Vec::new();
        if count >= 0 {
            for _ in 0..count {
                let dep: TypeIdentifierWithSize = reader.read_value()?;
                dependent_typeids.push(dep);
            }
        }

        Ok(Self {
            typeid_with_size,
            dependent_typeids,
        })
    }
}

impl<C: Context> Writable<C> for TypeIdentifierWithDependencies {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u32(self.content_size() as u32)?;
        writer.write_value(&self.typeid_with_size)?;
        writer.write_i32(self.dependent_typeids.len() as i32)?;
        for dep in &self.dependent_typeids {
            writer.write_value(dep)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInformation {
    pub minimal: TypeIdentifierWithDependencies,
    pub complete: TypeIdentifierWithDependencies,
}

const TYPEID_WITH_DEPS_MINIMAL: u32 = 0x1001;
const TYPEID_WITH_DEPS_COMPLETE: u32 = 0x1002;

impl<'a, C: Context> Readable<'a, C> for TypeInformation {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let _dheader = reader.read_u32()?;

        let mut minimal: Option<TypeIdentifierWithDependencies> = None;
        let mut complete: Option<TypeIdentifierWithDependencies> = None;

        loop {
            let emheader = match reader.read_u32() {
                Ok(v) => v,
                Err(_) => break,
            };

            let member_id = emheader >> 4;
            let lc = emheader & 0x07;

            let member_size = if lc < 4 {
                match lc {
                    0 => 1u32,
                    1 => 2,
                    2 => 4,
                    3 => 8,
                    _ => 0,
                }
            } else {
                reader.read_u32()?
            };

            match member_id {
                TYPEID_WITH_DEPS_MINIMAL => {
                    minimal = Some(reader.read_value()?);
                }
                TYPEID_WITH_DEPS_COMPLETE => {
                    complete = Some(reader.read_value()?);
                }
                _ => {
                    reader.skip_bytes(member_size as usize)?;
                }
            }

            if minimal.is_some() && complete.is_some() {
                break;
            }
        }

        let minimal = minimal.unwrap_or_else(|| TypeIdentifierWithDependencies {
            typeid_with_size: TypeIdentifierWithSize {
                type_id: TypeIdentifier::Primitive(TK_NONE),
                typeobject_serialized_size: 0,
            },
            dependent_typeids: Vec::new(),
        });
        let complete = complete.unwrap_or_else(|| TypeIdentifierWithDependencies {
            typeid_with_size: TypeIdentifierWithSize {
                type_id: TypeIdentifier::Primitive(TK_NONE),
                typeobject_serialized_size: 0,
            },
            dependent_typeids: Vec::new(),
        });

        Ok(Self { minimal, complete })
    }
}

impl<C: Context> Writable<C> for TypeInformation {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        let minimal_serialized_size = 4 + self.minimal.content_size();
        let complete_serialized_size = 4 + self.complete.content_size();

        // DHEADER: total body length (2 EMHEADERs + 2 sizes + content)
        let body_len = 4 + 4 + minimal_serialized_size + 4 + 4 + complete_serialized_size;
        writer.write_u32(body_len as u32)?;

        // EMHEADER for minimal (member_id=0x1001, lc=4 means next u32 is size)
        let emheader_minimal = (TYPEID_WITH_DEPS_MINIMAL << 4) | 4;
        writer.write_u32(emheader_minimal)?;
        writer.write_u32(minimal_serialized_size as u32)?;
        writer.write_value(&self.minimal)?;

        // EMHEADER for complete (member_id=0x1002, lc=4 means next u32 is size)
        let emheader_complete = (TYPEID_WITH_DEPS_COMPLETE << 4) | 4;
        writer.write_u32(emheader_complete)?;
        writer.write_u32(complete_serialized_size as u32)?;
        writer.write_value(&self.complete)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_identifier_primitive_round_trip() {
        for disc in [
            TK_BOOLEAN, TK_BYTE, TK_INT16, TK_INT32, TK_INT64, TK_UINT16, TK_UINT32, TK_UINT64,
            TK_FLOAT32, TK_FLOAT64, TK_INT8, TK_UINT8,
        ] {
            let ti = TypeIdentifier::Primitive(disc);
            let bytes = ti
                .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
                .unwrap();
            let ti2 = TypeIdentifier::read_from_buffer_with_ctx(
                speedy::Endianness::LittleEndian,
                &bytes,
            )
            .unwrap();
            assert_eq!(ti, ti2);
        }
    }

    #[test]
    fn type_identifier_string_small_round_trip() {
        let ti = TypeIdentifier::String8Small { bound: 255 };
        let bytes = ti
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let ti2 =
            TypeIdentifier::read_from_buffer_with_ctx(speedy::Endianness::LittleEndian, &bytes)
                .unwrap();
        assert_eq!(ti, ti2);
    }

    #[test]
    fn type_identifier_equivalence_hash_round_trip() {
        let hash: EquivalenceHash = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        let ti = TypeIdentifier::CompleteHash(hash);
        let bytes = ti
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let ti2 =
            TypeIdentifier::read_from_buffer_with_ctx(speedy::Endianness::LittleEndian, &bytes)
                .unwrap();
        assert_eq!(ti, ti2);
    }

    #[test]
    fn type_identifier_sequence_small_round_trip() {
        let ti = TypeIdentifier::PlainSequenceSmall {
            header: PlainCollectionHeader {
                equiv_kind: EK_COMPLETE,
                element_flags: 0,
            },
            bound: 0,
            element_identifier: Box::new(TypeIdentifier::Primitive(TK_INT32)),
        };
        let bytes = ti
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let ti2 =
            TypeIdentifier::read_from_buffer_with_ctx(speedy::Endianness::LittleEndian, &bytes)
                .unwrap();
        assert_eq!(ti, ti2);
    }

    #[test]
    fn type_identifier_with_size_round_trip() {
        let tws = TypeIdentifierWithSize {
            type_id: TypeIdentifier::CompleteHash([
                0xAA, 0xBB, 0xCC, 0xDD, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
                0x00,
            ]),
            typeobject_serialized_size: 128,
        };
        let bytes = tws
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let tws2 = TypeIdentifierWithSize::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &bytes,
        )
        .unwrap();
        assert_eq!(tws, tws2);
    }

    #[test]
    fn type_identifier_with_deps_round_trip() {
        let twd = TypeIdentifierWithDependencies {
            typeid_with_size: TypeIdentifierWithSize {
                type_id: TypeIdentifier::CompleteHash([1; 14]),
                typeobject_serialized_size: 256,
            },
            dependent_typeids: vec![
                TypeIdentifierWithSize {
                    type_id: TypeIdentifier::CompleteHash([2; 14]),
                    typeobject_serialized_size: 64,
                },
                TypeIdentifierWithSize {
                    type_id: TypeIdentifier::CompleteHash([3; 14]),
                    typeobject_serialized_size: 32,
                },
            ],
        };
        let bytes = twd
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let twd2 = TypeIdentifierWithDependencies::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &bytes,
        )
        .unwrap();
        assert_eq!(twd, twd2);
    }

    #[test]
    fn type_information_round_trip() {
        let ti = TypeInformation {
            minimal: TypeIdentifierWithDependencies {
                typeid_with_size: TypeIdentifierWithSize {
                    type_id: TypeIdentifier::MinimalHash([0x10; 14]),
                    typeobject_serialized_size: 100,
                },
                dependent_typeids: vec![],
            },
            complete: TypeIdentifierWithDependencies {
                typeid_with_size: TypeIdentifierWithSize {
                    type_id: TypeIdentifier::CompleteHash([0x20; 14]),
                    typeobject_serialized_size: 200,
                },
                dependent_typeids: vec![TypeIdentifierWithSize {
                    type_id: TypeIdentifier::CompleteHash([0x30; 14]),
                    typeobject_serialized_size: 50,
                }],
            },
        };
        let bytes = ti
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let ti2 = TypeInformation::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &bytes,
        )
        .unwrap();
        assert_eq!(ti, ti2);
    }
}
