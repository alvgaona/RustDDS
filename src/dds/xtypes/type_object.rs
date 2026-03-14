use speedy::{Context, Readable, Reader, Writable, Writer};

use super::type_identifier::{TypeIdentifier, EK_COMPLETE, EK_MINIMAL};

use crate::serialization::speedy_pl_cdr_helpers::StringWithNul;

// DDS-XTypes v1.3 Type kind values for CompleteTypeObject/MinimalTypeObject
pub const TK_ALIAS: u8 = 0x01;
pub const TK_ENUM: u8 = 0x02;
pub const TK_BITMASK: u8 = 0x03;
pub const TK_ANNOTATION: u8 = 0x04;
pub const TK_STRUCTURE: u8 = 0x05;
pub const TK_UNION: u8 = 0x06;
pub const TK_BITSET: u8 = 0x07;
pub const TK_SEQUENCE: u8 = 0x08;
pub const TK_ARRAY: u8 = 0x09;
pub const TK_MAP: u8 = 0x0A;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeObject {
    Complete(CompleteTypeObject),
    Minimal(MinimalTypeObject),
}

impl<'a, C: Context> Readable<'a, C> for TypeObject {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let disc = reader.read_u8()?;
        match disc {
            EK_COMPLETE => Ok(TypeObject::Complete(reader.read_value()?)),
            EK_MINIMAL => Ok(TypeObject::Minimal(reader.read_value()?)),
            _ => Ok(TypeObject::Complete(CompleteTypeObject::Unknown)),
        }
    }
}

impl<C: Context> Writable<C> for TypeObject {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        match self {
            TypeObject::Complete(c) => {
                writer.write_u8(EK_COMPLETE)?;
                writer.write_value(c)?;
            }
            TypeObject::Minimal(m) => {
                writer.write_u8(EK_MINIMAL)?;
                writer.write_value(m)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompleteTypeObject {
    Struct(CompleteStructType),
    Enum(CompleteEnumeratedType),
    Alias(CompleteAliasType),
    Sequence(CompleteSequenceType),
    Array(CompleteArrayType),
    Union(CompleteUnionType),
    Unknown,
}

impl<'a, C: Context> Readable<'a, C> for CompleteTypeObject {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let disc = reader.read_u8()?;
        match disc {
            TK_STRUCTURE => Ok(CompleteTypeObject::Struct(reader.read_value()?)),
            TK_ENUM => Ok(CompleteTypeObject::Enum(reader.read_value()?)),
            TK_ALIAS => Ok(CompleteTypeObject::Alias(reader.read_value()?)),
            TK_SEQUENCE => Ok(CompleteTypeObject::Sequence(reader.read_value()?)),
            TK_ARRAY => Ok(CompleteTypeObject::Array(reader.read_value()?)),
            TK_UNION => Ok(CompleteTypeObject::Union(reader.read_value()?)),
            _ => Ok(CompleteTypeObject::Unknown),
        }
    }
}

impl<C: Context> Writable<C> for CompleteTypeObject {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        match self {
            CompleteTypeObject::Struct(s) => {
                writer.write_u8(TK_STRUCTURE)?;
                writer.write_value(s)?;
            }
            CompleteTypeObject::Enum(e) => {
                writer.write_u8(TK_ENUM)?;
                writer.write_value(e)?;
            }
            CompleteTypeObject::Alias(a) => {
                writer.write_u8(TK_ALIAS)?;
                writer.write_value(a)?;
            }
            CompleteTypeObject::Sequence(s) => {
                writer.write_u8(TK_SEQUENCE)?;
                writer.write_value(s)?;
            }
            CompleteTypeObject::Array(a) => {
                writer.write_u8(TK_ARRAY)?;
                writer.write_value(a)?;
            }
            CompleteTypeObject::Union(u) => {
                writer.write_u8(TK_UNION)?;
                writer.write_value(u)?;
            }
            CompleteTypeObject::Unknown => {
                writer.write_u8(0xFF)?;
            }
        }
        Ok(())
    }
}

// Minimal type object — we store it as raw bytes for now since we primarily
// need the Complete variant for type introspection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimalTypeObject {
    pub raw: Vec<u8>,
}

impl<'a, C: Context> Readable<'a, C> for MinimalTypeObject {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let _disc = reader.read_u8()?;
        // Read remaining bytes - this is a simplification; in practice
        // we'd need the full DHEADER-bounded parsing
        Ok(Self { raw: Vec::new() })
    }
}

impl<C: Context> Writable<C> for MinimalTypeObject {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u8(0xFF)?;
        Ok(())
    }
}

// --- Complete Struct Type ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteStructType {
    pub struct_flags: u16,
    pub header: CompleteStructHeader,
    pub members: Vec<CompleteStructMember>,
}

impl<'a, C: Context> Readable<'a, C> for CompleteStructType {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let struct_flags = reader.read_u16()?;
        let header: CompleteStructHeader = reader.read_value()?;
        let count = reader.read_u32()?;
        let mut members = Vec::with_capacity(count as usize);
        for _ in 0..count {
            members.push(reader.read_value()?);
        }
        Ok(Self {
            struct_flags,
            header,
            members,
        })
    }
}

impl<C: Context> Writable<C> for CompleteStructType {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u16(self.struct_flags)?;
        writer.write_value(&self.header)?;
        writer.write_u32(self.members.len() as u32)?;
        for m in &self.members {
            writer.write_value(m)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteStructHeader {
    pub base_type: TypeIdentifier,
    pub type_name: String,
}

impl<'a, C: Context> Readable<'a, C> for CompleteStructHeader {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let base_type: TypeIdentifier = reader.read_value()?;
        let type_name: StringWithNul = reader.read_value()?;
        Ok(Self {
            base_type,
            type_name: type_name.into(),
        })
    }
}

impl<C: Context> Writable<C> for CompleteStructHeader {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.base_type)?;
        writer.write_value(&StringWithNul::from(self.type_name.clone()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteStructMember {
    pub common: CommonStructMember,
    pub name: String,
}

impl<'a, C: Context> Readable<'a, C> for CompleteStructMember {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let common: CommonStructMember = reader.read_value()?;
        let name: StringWithNul = reader.read_value()?;
        Ok(Self {
            common,
            name: name.into(),
        })
    }
}

impl<C: Context> Writable<C> for CompleteStructMember {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.common)?;
        writer.write_value(&StringWithNul::from(self.name.clone()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonStructMember {
    pub member_id: u32,
    pub member_flags: u16,
    pub member_type_id: TypeIdentifier,
}

impl<'a, C: Context> Readable<'a, C> for CommonStructMember {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let member_id = reader.read_u32()?;
        let member_flags = reader.read_u16()?;
        let member_type_id: TypeIdentifier = reader.read_value()?;
        Ok(Self {
            member_id,
            member_flags,
            member_type_id,
        })
    }
}

impl<C: Context> Writable<C> for CommonStructMember {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u32(self.member_id)?;
        writer.write_u16(self.member_flags)?;
        writer.write_value(&self.member_type_id)?;
        Ok(())
    }
}

// --- Complete Enumerated Type ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteEnumeratedType {
    pub enum_flags: u16,
    pub bit_bound: u16,
    pub type_name: String,
    pub literals: Vec<CompleteEnumeratedLiteral>,
}

impl<'a, C: Context> Readable<'a, C> for CompleteEnumeratedType {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let enum_flags = reader.read_u16()?;
        let bit_bound = reader.read_u16()?;
        let type_name: StringWithNul = reader.read_value()?;
        let count = reader.read_u32()?;
        let mut literals = Vec::with_capacity(count as usize);
        for _ in 0..count {
            literals.push(reader.read_value()?);
        }
        Ok(Self {
            enum_flags,
            bit_bound,
            type_name: type_name.into(),
            literals,
        })
    }
}

impl<C: Context> Writable<C> for CompleteEnumeratedType {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u16(self.enum_flags)?;
        writer.write_u16(self.bit_bound)?;
        writer.write_value(&StringWithNul::from(self.type_name.clone()))?;
        writer.write_u32(self.literals.len() as u32)?;
        for l in &self.literals {
            writer.write_value(l)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteEnumeratedLiteral {
    pub value: i32,
    pub flags: u16,
    pub name: String,
}

impl<'a, C: Context> Readable<'a, C> for CompleteEnumeratedLiteral {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let value = reader.read_i32()?;
        let flags = reader.read_u16()?;
        let name: StringWithNul = reader.read_value()?;
        Ok(Self {
            value,
            flags,
            name: name.into(),
        })
    }
}

impl<C: Context> Writable<C> for CompleteEnumeratedLiteral {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_i32(self.value)?;
        writer.write_u16(self.flags)?;
        writer.write_value(&StringWithNul::from(self.name.clone()))?;
        Ok(())
    }
}

// --- Complete Alias Type ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteAliasType {
    pub alias_flags: u16,
    pub type_name: String,
    pub body_flags: u16,
    pub related_type: TypeIdentifier,
}

impl<'a, C: Context> Readable<'a, C> for CompleteAliasType {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let alias_flags = reader.read_u16()?;
        let type_name: StringWithNul = reader.read_value()?;
        let body_flags = reader.read_u16()?;
        let related_type: TypeIdentifier = reader.read_value()?;
        Ok(Self {
            alias_flags,
            type_name: type_name.into(),
            body_flags,
            related_type,
        })
    }
}

impl<C: Context> Writable<C> for CompleteAliasType {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u16(self.alias_flags)?;
        writer.write_value(&StringWithNul::from(self.type_name.clone()))?;
        writer.write_u16(self.body_flags)?;
        writer.write_value(&self.related_type)?;
        Ok(())
    }
}

// --- Complete Sequence Type ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteSequenceType {
    pub collection_flags: u16,
    pub bound: u32,
    pub element_flags: u16,
    pub element_type: TypeIdentifier,
}

impl<'a, C: Context> Readable<'a, C> for CompleteSequenceType {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let collection_flags = reader.read_u16()?;
        let bound = reader.read_u32()?;
        let element_flags = reader.read_u16()?;
        let element_type: TypeIdentifier = reader.read_value()?;
        Ok(Self {
            collection_flags,
            bound,
            element_flags,
            element_type,
        })
    }
}

impl<C: Context> Writable<C> for CompleteSequenceType {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u16(self.collection_flags)?;
        writer.write_u32(self.bound)?;
        writer.write_u16(self.element_flags)?;
        writer.write_value(&self.element_type)?;
        Ok(())
    }
}

// --- Complete Array Type ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteArrayType {
    pub collection_flags: u16,
    pub bound: Vec<u32>,
    pub element_flags: u16,
    pub element_type: TypeIdentifier,
}

impl<'a, C: Context> Readable<'a, C> for CompleteArrayType {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let collection_flags = reader.read_u16()?;
        let bound_count = reader.read_u32()?;
        let mut bound = Vec::with_capacity(bound_count as usize);
        for _ in 0..bound_count {
            bound.push(reader.read_u32()?);
        }
        let element_flags = reader.read_u16()?;
        let element_type: TypeIdentifier = reader.read_value()?;
        Ok(Self {
            collection_flags,
            bound,
            element_flags,
            element_type,
        })
    }
}

impl<C: Context> Writable<C> for CompleteArrayType {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u16(self.collection_flags)?;
        writer.write_u32(self.bound.len() as u32)?;
        for b in &self.bound {
            writer.write_u32(*b)?;
        }
        writer.write_u16(self.element_flags)?;
        writer.write_value(&self.element_type)?;
        Ok(())
    }
}

// --- Complete Union Type ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteUnionType {
    pub union_flags: u16,
    pub type_name: String,
    pub discriminator_flags: u16,
    pub discriminator_type: TypeIdentifier,
    pub members: Vec<CompleteUnionMember>,
}

impl<'a, C: Context> Readable<'a, C> for CompleteUnionType {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let union_flags = reader.read_u16()?;
        let type_name: StringWithNul = reader.read_value()?;
        let discriminator_flags = reader.read_u16()?;
        let discriminator_type: TypeIdentifier = reader.read_value()?;
        let count = reader.read_u32()?;
        let mut members = Vec::with_capacity(count as usize);
        for _ in 0..count {
            members.push(reader.read_value()?);
        }
        Ok(Self {
            union_flags,
            type_name: type_name.into(),
            discriminator_flags,
            discriminator_type,
            members,
        })
    }
}

impl<C: Context> Writable<C> for CompleteUnionType {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u16(self.union_flags)?;
        writer.write_value(&StringWithNul::from(self.type_name.clone()))?;
        writer.write_u16(self.discriminator_flags)?;
        writer.write_value(&self.discriminator_type)?;
        writer.write_u32(self.members.len() as u32)?;
        for m in &self.members {
            writer.write_value(m)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteUnionMember {
    pub member_id: u32,
    pub member_flags: u16,
    pub member_type_id: TypeIdentifier,
    pub label_count: u32,
    pub labels: Vec<i32>,
    pub name: String,
}

impl<'a, C: Context> Readable<'a, C> for CompleteUnionMember {
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let member_id = reader.read_u32()?;
        let member_flags = reader.read_u16()?;
        let member_type_id: TypeIdentifier = reader.read_value()?;
        let label_count = reader.read_u32()?;
        let mut labels = Vec::with_capacity(label_count as usize);
        for _ in 0..label_count {
            labels.push(reader.read_i32()?);
        }
        let name: StringWithNul = reader.read_value()?;
        Ok(Self {
            member_id,
            member_flags,
            member_type_id,
            label_count,
            labels,
            name: name.into(),
        })
    }
}

impl<C: Context> Writable<C> for CompleteUnionMember {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u32(self.member_id)?;
        writer.write_u16(self.member_flags)?;
        writer.write_value(&self.member_type_id)?;
        writer.write_u32(self.label_count)?;
        for l in &self.labels {
            writer.write_i32(*l)?;
        }
        writer.write_value(&StringWithNul::from(self.name.clone()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_struct_member_round_trip() {
        let member = CompleteStructMember {
            common: CommonStructMember {
                member_id: 0,
                member_flags: 0,
                member_type_id: TypeIdentifier::Primitive(
                    super::super::type_identifier::TK_INT32,
                ),
            },
            name: "x".to_string(),
        };
        let bytes = member
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let member2 = CompleteStructMember::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &bytes,
        )
        .unwrap();
        assert_eq!(member, member2);
    }

    #[test]
    fn complete_struct_type_round_trip() {
        let st = CompleteStructType {
            struct_flags: 0,
            header: CompleteStructHeader {
                base_type: TypeIdentifier::Primitive(
                    super::super::type_identifier::TK_NONE,
                ),
                type_name: "geometry_msgs::msg::Vector3".to_string(),
            },
            members: vec![
                CompleteStructMember {
                    common: CommonStructMember {
                        member_id: 0,
                        member_flags: 0,
                        member_type_id: TypeIdentifier::Primitive(
                            super::super::type_identifier::TK_FLOAT64,
                        ),
                    },
                    name: "x".to_string(),
                },
                CompleteStructMember {
                    common: CommonStructMember {
                        member_id: 1,
                        member_flags: 0,
                        member_type_id: TypeIdentifier::Primitive(
                            super::super::type_identifier::TK_FLOAT64,
                        ),
                    },
                    name: "y".to_string(),
                },
                CompleteStructMember {
                    common: CommonStructMember {
                        member_id: 2,
                        member_flags: 0,
                        member_type_id: TypeIdentifier::Primitive(
                            super::super::type_identifier::TK_FLOAT64,
                        ),
                    },
                    name: "z".to_string(),
                },
            ],
        };
        let bytes = st
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let st2 = CompleteStructType::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &bytes,
        )
        .unwrap();
        assert_eq!(st, st2);
    }

    #[test]
    fn complete_enum_type_round_trip() {
        let et = CompleteEnumeratedType {
            enum_flags: 0,
            bit_bound: 32,
            type_name: "MyEnum".to_string(),
            literals: vec![
                CompleteEnumeratedLiteral {
                    value: 0,
                    flags: 0,
                    name: "VALUE_A".to_string(),
                },
                CompleteEnumeratedLiteral {
                    value: 1,
                    flags: 0,
                    name: "VALUE_B".to_string(),
                },
            ],
        };
        let bytes = et
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let et2 = CompleteEnumeratedType::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &bytes,
        )
        .unwrap();
        assert_eq!(et, et2);
    }

    #[test]
    fn type_object_complete_struct_round_trip() {
        let to = TypeObject::Complete(CompleteTypeObject::Struct(CompleteStructType {
            struct_flags: 0,
            header: CompleteStructHeader {
                base_type: TypeIdentifier::Primitive(
                    super::super::type_identifier::TK_NONE,
                ),
                type_name: "std_msgs::msg::String".to_string(),
            },
            members: vec![CompleteStructMember {
                common: CommonStructMember {
                    member_id: 0,
                    member_flags: 0,
                    member_type_id: TypeIdentifier::String8Small { bound: 0 },
                },
                name: "data".to_string(),
            }],
        }));
        let bytes = to
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .unwrap();
        let to2 =
            TypeObject::read_from_buffer_with_ctx(speedy::Endianness::LittleEndian, &bytes)
                .unwrap();
        assert_eq!(to, to2);
    }
}
