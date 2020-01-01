use super::*;

/// The way to look up an enum within the collection of all enums.
///
/// Most enums have api `None`, indicating that they have no restriction and use
/// the same definition in all APIs. A very select few enums have an api value
/// defined because the same name is given _different_ values depending on the
/// GL API in use.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct EnumKey {
  /// Enum name
  pub name: String,
  /// Enum availability (eg: "gl", "gles2", `None` == always)
  pub api: Option<String>,
}

/// The value variations that an Enum name can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumValue {
  /// A `GLenum`
  Enum(u32),
  /// A `GLbitfield`
  Bitmask(u32),
  /// A `u64`
  ///
  /// Technically an `unsigned long long` that is "Tagged as uint64".
  ULL(u64),
}

/// Allows properly displaying an enum key/value pair.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct EnumDisplay<'a> {
  pub key: &'a EnumKey,
  pub value: &'a EnumValue,
}
impl core::fmt::Display for EnumDisplay<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    // trim off the `GL_` prefix if the alternate is requested.
    let the_name =
      if f.alternate() { &self.key.name[3..] } else { &self.key.name };
    match self.value {
      EnumValue::Enum(num) => write!(
        f,
        "pub const {name}: GLenum = 0x{num:X};",
        name = the_name,
        num = num
      ),
      EnumValue::Bitmask(mask) => write!(
        f,
        "pub const {name}: GLbitfield = 0x{mask:08X};",
        name = the_name,
        mask = mask
      ),
      EnumValue::ULL(ull) => write!(
        f,
        "pub const {name}: u64 = 0x{ull:X};",
        name = the_name,
        ull = ull
      ),
    }
  }
}

/// A map of enum keys to values
#[derive(Debug, Default, Clone)]
pub struct Enums(pub(crate) HashMap<EnumKey, EnumValue>);

/// Grabs an `enums` tag from the iterator.
///
/// * `is_bitmask`: This is part of the tag attributes, you provide it.
/// * `group`: Also a tag attribute. If you provide a reference here all enums
///   collected for this tag will also be added into the group.
#[must_use]
#[allow(clippy::collapsible_if)]
pub fn pull_enums(
  it: &mut XmlIterator<'_>,
  enums: &mut Enums,
  is_bitmask: bool,
  mut group: Option<&mut HashSet<String>>,
) -> Option<()> {
  loop {
    match it.next()? {
      EndTag { name: "enums" } => return Some(()),
      EmptyTag { name: "enum", attrs } => {
        let mut name = None;
        let mut value = None;
        //let mut alias = None;
        let mut api = None;
        let mut is_ull = false;
        for (k, v) in AttributeIterator::new(attrs) {
          match k {
            "name" => name = Some(v),
            "value" => value = Some(v),
            "alias" => (), //alias = Some(v),
            "comment" => (),
            "type" => is_ull = v == "ull",
            "api" => api = Some(v.to_owned()),
            _ => panic!("unexpected key> {}; {}", k, attrs),
          }
        }
        let name = name.unwrap().to_owned();
        let value = value.unwrap();
        let val: EnumValue = if is_ull {
          EnumValue::ULL(u64::from_str_radix(&value[2..], 16).unwrap())
        } else if value.contains('x') || value.contains('X') {
          if is_bitmask {
            EnumValue::Bitmask(u32::from_str_radix(&value[2..], 16).unwrap())
          } else {
            EnumValue::Enum(u32::from_str_radix(&value[2..], 16).unwrap())
          }
        } else if value.contains('-') {
          if is_bitmask {
            EnumValue::Bitmask(i32::from_str_radix(&value, 16).unwrap() as u32)
          } else {
            EnumValue::Enum(i32::from_str_radix(&value, 16).unwrap() as u32)
          }
        } else {
          if is_bitmask {
            EnumValue::Bitmask(u32::from_str_radix(&value, 10).unwrap())
          } else {
            EnumValue::Enum(u32::from_str_radix(&value, 10).unwrap())
          }
        };
        let key = EnumKey { name, api: api.clone() };
        if enums.0.contains_key(&key) {
          let old = *enums.0.get(&key).unwrap();
          let new = val;
          if old != new {
            panic!(
              "key overwrite: key: {:?}, old: {:?}, new: {:?}",
              key, old, new
            );
          }
        } else {
          if let Some(group) = group.as_mut() {
            group.insert(key.name.clone());
          }
          enums.0.insert(key, val);
        }
        // if let Some(alias) = alias {
        //   let name = alias.to_owned();
        //   let key = EnumKey { name, api: api.clone() };
        //   if enums.0.contains_key(&key) {
        //     let old = *enums.0.get(&key).unwrap();
        //     let new = val;
        //     if old != new {
        //       panic!(
        //         "key overwrite: key: {:?}, old: {:?}, new: {:?}",
        //         key, old, new
        //       );
        //     }
        //   } else {
        //     if let Some(group) = group.as_mut() {
        //       group.insert(key.name.clone());
        //     }
        //     enums.0.insert(key, val);
        //   }
        // }
      }
      EmptyTag { name: "unused", attrs } => {
        // TODO: We should check if the `unused` tag is somehow used despite the
        // name, because programming is that bad at naming sometimes.
        let _ = attrs;
      }
      other => panic!("unexpected> {:?}", other),
    }
  }
}
