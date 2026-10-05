---
sealmap: 2
source: crates/sealmap-model/src/sym.rs
module: "sym:cargo sealmap_model . sym/"
language: rust
source_hash: blake3:c6f426eb9516620eace067b563ddbec99474694a54e5a4e42fdd46b976c9edcc
lines: 1168
fragments: 32
---
# `sym:cargo sealmap_model . sym/` · crates/sealmap-model/src/sym.rs
> The `sym:` symbol-id grammar.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_model__sym___tDescSpan["DescSpan#lt;'a#gt;"] {
    <<struct>>
    -end: usize
    -name: &'a str
    -kind: u8
    -disambiguator: &'a str
  }
  class sealmap_model__sym___tDescriptor["Descriptor"] {
    <<struct>>
    -name: String
    -suffix: Suffix
    +r#35;macro(name: impl Into#lt;String#gt;) Self
    +r#35;type(name: impl Into#lt;String#gt;) Self
    +meta(name: impl Into#lt;String#gt;) Self
    +method(name: impl Into#lt;String#gt;) Self
    +name(&self) &str
    +namespace(name: impl Into#lt;String#gt;) Self
    +new(name: impl Into#lt;String#gt;, suffix: Suffix) Result#lt;Self, IdError#gt;
    +parameter(name: impl Into#lt;String#gt;) Self
    +suffix(&self) &Suffix
    +term(name: impl Into#lt;String#gt;) Self
    +type_parameter(name: impl Into#lt;String#gt;) Self
    -write(&self, out: &mut String)
  }
  class sealmap_model__sym___tDescriptorKind["DescriptorKind"] {
    <<enum>>
    Namespace
    Type
    Term
    Method
    TypeParameter
    Parameter
    Meta
    Macro
  }
  class sealmap_model__sym___tDescriptorView["DescriptorView#lt;'a#gt;"] {
    <<struct>>
    +name: Cow#lt;'a, str#gt;
    +kind: DescriptorKind
    +disambiguator: Option#lt;&'a str#gt;
  }
  class sealmap_model__sym___tGlobalView["GlobalView#lt;'a#gt;"] {
    <<struct>>
    +manager: &'a str
    +package: Cow#lt;'a, str#gt;
    +version: Option#lt;Cow#lt;'a, str#gt;#gt;
    +descriptors: Vec#lt;DescriptorView#lt;'a#gt;#gt;
  }
  class sealmap_model__sym___tIdError["IdError"] {
    <<enum>>
    MissingPrefix
    Manager#40;String#41;
    Field#40;String#41;
    Disambiguator#40;String#41;
    EmptyPath
    Syntax#123; #35;[doc = #quot; Byte offset into the input.#quot;] at: usize, #35;[…
  }
  class sealmap_model__sym___tIdView["IdView#lt;'a#gt;"] {
    <<enum>>
    Global#40;GlobalView#lt;'a#gt;#41;
    Path#40;Vec#lt;Cow#lt;'a, str#gt;#gt;#41;
    Unresolved#40;Cow#lt;'a, str#gt;#41;
  }
  class sealmap_model__sym___tPackage["Package"] {
    <<struct>>
    -manager: String
    -name: String
    -version: Version
    +current(manager: impl Into#lt;String#gt;, name: impl Into#lt;String#gt;) Result#lt;Self, IdError#gt;
    +manager(&self) &str
    +name(&self) &str
    +new(manager: impl Into#lt;String#gt;, name: impl Into#lt;String#gt;, version: Version) Result#lt;Self, IdError#gt;
    +version(&self) &Version
  }
  class sealmap_model__sym___tParser["Parser#lt;'a#gt;"] {
    <<struct>>
    -s: &'a str
    -pos: usize
    -descriptor(&mut self) Result#lt;Descriptor, IdError#gt;
    -done(&self) Result#lt;#40;#41;, IdError#gt;
    -eat(&mut self, lit: &str) bool
    -err(&self, why: &'static str) IdError
    -expect(&mut self, lit: &str, why: &'static str) Result#lt;#40;#41;, IdError#gt;
    -field(&mut self) Result#lt;String, IdError#gt;
    -id(mut self) Result#lt;Repr, IdError#gt;
    -name(&mut self) Result#lt;String, IdError#gt;
    -peek(&self) Option#lt;char#gt;
    -rest(&self) &str
  }
  class sealmap_model__sym___tRepr["Repr"] {
    <<enum>>
    Global#123; package: Package, descriptors: Vec#lt;Descriptor#gt; #125;
    Path#40;Vec#lt;String#gt;#41;
    Unresolved#40;String#41;
  }
  class sealmap_model__sym___tShape["Shape#lt;'a#gt;"] {
    <<enum>>
    Global#123; manager: &'a str, name: &'a str, version: &'a str, de…
    Path#40;&'a str#41;
    Unresolved#40;&'a str#41;
    -of(text: &'a str) Self
  }
  class sealmap_model__sym___tSuffix["Suffix"] {
    <<enum>>
    Namespace
    Type
    Term
    Method#123; #35;[doc = #quot; Overload disambiguator, made of simple char…
    TypeParameter
    Parameter
    Meta
    Macro
  }
  class sealmap_model__sym___tSymbolId["SymbolId"] {
    <<struct>>
    -0: Arc#lt;str#gt;
    +FromStr::from_str(s: &str) Result#lt;Self, Self::Err#gt;
    +Serialize::serialize(&self, serializer: S) Result#lt;S::Ok, S::Error#gt;
    +Deserialize#lt;'de#gt;::deserialize(deserializer: D) Result#lt;Self, D::Error#gt;
    +Debug::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +Display::fmt(&self, f: &mut fmt::Formatter#lt;'_#gt;) fmt::Result
    +as_str(&self) &str
    +child(&self, descriptor: Descriptor) Option#lt;SymbolId#gt;
    +descriptors(&self) Vec#lt;Descriptor#gt;
    +display_path(&self) String
    +extend_path(&self, segment: &str) SymbolId
    -from_repr(repr: &Repr) Self
    +global(package: Package, descriptors: Vec#lt;Descriptor#gt;) Self
    +is_global(&self) bool
    +is_unresolved(&self) bool
    +last(&self) Option#lt;Descriptor#gt;
    ~min_value(crate) Self
    +name(&self) Cow#lt;'_, str#gt;
    +names(&self) Vec#lt;Cow#lt;'_, str#gt;#gt;
    +package(&self) Option#lt;Package#gt;
    +package_root(package: Package) Self
    +parent(&self) Option#lt;SymbolId#gt;
    +parse(text: &str) Result#lt;Self, IdError#gt;
    +path(segments: I) Result#lt;Self, IdError#gt;
    -repr(&self) Repr
    +root(&self) Option#lt;Cow#lt;'_, str#gt;#gt;
    +segments(&self) Option#lt;Vec#lt;String#gt;#gt;
    -shape(&self) Shape#lt;'_#gt;
    +unresolved(name: impl Into#lt;String#gt;) Self
    +view(&self) IdView#lt;'_#gt;
  }
  class sealmap_model__sym___tVersion["Version"] {
    <<enum>>
    Current
    Release#40;String#41;
  }
  class sealmap_model__sym["sealmap_model::sym"] {
    <<module>>
    -const EXTERN: &str
    +const SYM_PREFIX: &str
    -check_field(f: &str) Result#lt;#40;#41;, IdError#gt;
    -check_manager(m: &str) Result#lt;#40;#41;, IdError#gt;
    -is_simple(c: char) bool
    -print(repr: &Repr) String
    -raw_name_len(s: &str) usize
    -scan_descriptors(s: &str) Vec#lt;DescSpan#lt;'_#gt;#gt;
    -scan_path(s: &str) Vec#lt;&str#gt;
    -unfield(raw: &str) Cow#lt;'_, str#gt;
    -unquote(raw: &str) Cow#lt;'_, str#gt;
    -write_field(field: &str, out: &mut String)
    -write_name(name: &str, out: &mut String)
  }
  class _serde__Deserialize["serde::Deserialize"] {
    <<external>>
  }
  class _serde__Serialize["serde::Serialize"] {
    <<external>>
  }
  sealmap_model__sym ..> sealmap_model__sym___tDescSpan
  sealmap_model__sym ..> sealmap_model__sym___tIdError
  sealmap_model__sym ..> sealmap_model__sym___tRepr
  sealmap_model__sym___tDescriptor ..> sealmap_model__sym___tIdError
  sealmap_model__sym___tDescriptor *-- sealmap_model__sym___tSuffix : suffix
  sealmap_model__sym___tDescriptorView *-- sealmap_model__sym___tDescriptorKind : kind
  sealmap_model__sym___tGlobalView o-- sealmap_model__sym___tDescriptorView : descriptors
  sealmap_model__sym___tIdView *-- sealmap_model__sym___tGlobalView : Global
  sealmap_model__sym___tPackage ..> sealmap_model__sym___tIdError
  sealmap_model__sym___tPackage *-- sealmap_model__sym___tVersion : version
  sealmap_model__sym___tParser ..> sealmap_model__sym___tDescriptor
  sealmap_model__sym___tParser ..> sealmap_model__sym___tIdError
  sealmap_model__sym___tParser ..> sealmap_model__sym___tRepr
  sealmap_model__sym___tRepr o-- sealmap_model__sym___tDescriptor : Global
  sealmap_model__sym___tRepr o-- sealmap_model__sym___tPackage : Global
  sealmap_model__sym___tSymbolId ..> sealmap_model__sym___tDescriptor
  sealmap_model__sym___tSymbolId ..> sealmap_model__sym___tIdError
  sealmap_model__sym___tSymbolId ..> sealmap_model__sym___tIdView
  sealmap_model__sym___tSymbolId ..> sealmap_model__sym___tPackage
  sealmap_model__sym___tSymbolId ..> sealmap_model__sym___tRepr
  sealmap_model__sym___tSymbolId ..> sealmap_model__sym___tShape
  sealmap_model__sym___tSymbolId ..|> _serde__Deserialize
  sealmap_model__sym___tSymbolId ..|> _serde__Serialize
```

## `sym:cargo sealmap_model . sym/Package#new().`
`pub fn new(manager: impl Into<String>, name: impl Into<String>, version: Version) -> Result<Self, IdError>` · L220-L236
> A package with an explicit version.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tPackage as Package
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tPackage->>sealmap_model__sym: check_manager(&manager)?
  sealmap_model__sym___tPackage->>sealmap_model__sym: check_field(&name)?
  opt name == #quot;.#quot;
    Note over sealmap_model__sym___tPackage: return Err(IdError::Field(name))
  end
  opt let Version::Release(v) = &version
    sealmap_model__sym___tPackage->>sealmap_model__sym: check_field(v)?
    opt v == #quot;.#quot;
      Note over sealmap_model__sym___tPackage: return Err(IdError::Field(v.clone()))
    end
  end
```

## `sym:cargo sealmap_model . sym/Descriptor#write().`
`fn write(&self, out: &mut String)` · L335-L364
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tDescriptor as Descriptor
  participant sealmap_model__sym as sym mod
  alt Suffix::TypeParameter
    sealmap_model__sym___tDescriptor->>sealmap_model__sym: write_name(&self.name, out)
  else Suffix::Parameter
    sealmap_model__sym___tDescriptor->>sealmap_model__sym: write_name(&self.name, out)
  else other
    sealmap_model__sym___tDescriptor->>sealmap_model__sym: write_name(&self.name, out)
  end
```

## `sym:cargo sealmap_model . sym/SymbolId#from_repr().`
`fn from_repr(repr: &Repr) -> Self` · L399-L401
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tSymbolId->>sealmap_model__sym: print(repr)
```

## `sym:cargo sealmap_model . sym/SymbolId#parse().`
`pub fn parse(text: &str) -> Result<Self, IdError>` · L432-L444
> Parse the canonical form.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tSymbolId->>sealmap_model__sym: print(&repr)
  opt print(&repr) != text
    Note over sealmap_model__sym___tSymbolId: return Err(IdError::Syntax { at: 0, why: #quot;not in canoni…
  end
```

## `sym:cargo sealmap_model . sym/SymbolId#shape().`
`fn shape(&self) -> Shape<'_>` · L460-L462
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym___tShape as Shape
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tShape: Shape::of(&self.0)
```

## `sym:cargo sealmap_model . sym/SymbolId#is_global().`
`pub fn is_global(&self) -> bool` · L464-L467
> `true` for global (kind-explicit) ids.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
```

## `sym:cargo sealmap_model . sym/SymbolId#is_unresolved().`
`pub fn is_unresolved(&self) -> bool` · L469-L472
> `true` for [`SymbolId::unresolved`] ids.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
```

## `sym:cargo sealmap_model . sym/SymbolId#package().`
`pub fn package(&self) -> Option<Package>` · L474-L480
> The package of a global id.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: repr()
```

## `sym:cargo sealmap_model . sym/SymbolId#descriptors().`
`pub fn descriptors(&self) -> Vec<Descriptor>` · L482-L488
> The descriptors of a global id (empty for the other forms).
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: repr()
```

## `sym:cargo sealmap_model . sym/SymbolId#segments().`
`pub fn segments(&self) -> Option<Vec<String>>` · L490-L496
> The segments of a path id.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: repr()
```

## `sym:cargo sealmap_model . sym/SymbolId#last().`
`pub fn last(&self) -> Option<Descriptor>` · L498-L501
> The last descriptor of a global id.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: descriptors()
```

## `sym:cargo sealmap_model . sym/SymbolId#name().`
`pub fn name(&self) -> Cow<'_, str>` · L503-L519
> The short name: the last descriptor's name, the package name of a package root, the last path segment, or the unresolved method name.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
  alt Shape::Global { name, descriptors,..}
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_descriptors(descriptors)
    opt via map_or_else
      sealmap_model__sym___tSymbolId->>sealmap_model__sym: unfield(name)
    end
    opt via map_or_else
      sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(d.name)
    end
  else Shape::Path(body)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_path(body)
    opt via map_or
      sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(s)
    end
  else Shape::Unresolved(raw)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(raw)
  end
```

## `sym:cargo sealmap_model . sym/SymbolId#root().`
`pub fn root(&self) -> Option<Cow<'_, str>>` · L521-L529
> The first name: the package name, or the first path segment.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
  alt Shape::Global { name,..}
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: unfield(name)
  else Shape::Path(body)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_path(body)
    opt via map
      sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(s)
    end
  end
```

## `sym:cargo sealmap_model . sym/SymbolId#parent().`
`pub fn parent(&self) -> Option<SymbolId>` · L531-L572
> The enclosing definition.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
  alt Shape::Global { descriptors,..}
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_descriptors(descriptors)
  else Shape::Path(body)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_path(body)
    opt segs.len()<2
      Note over sealmap_model__sym___tSymbolId: return None
    end
  end
```

## `sym:cargo sealmap_model . sym/SymbolId#child().`
`pub fn child(&self, descriptor: Descriptor) -> Option<SymbolId>` · L574-L585
> Append a descriptor to a global id.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym___tDescriptor as Descriptor
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
  opt let-else
    Note over sealmap_model__sym___tSymbolId: return None
  end
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tDescriptor: write(&out)
```

## `sym:cargo sealmap_model . sym/SymbolId#names().`
`pub fn names(&self) -> Vec<Cow<'_, str>>` · L587-L597
> Every name in order: package and descriptor names for a global id, the segments of a path id, the name of an unresolved id.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
  alt Shape::Global { name, descriptors,..}
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: unfield(name)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_descriptors(descriptors)
    opt via map
      sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(d.name)
    end
  else Shape::Path(body)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_path(body)
  else Shape::Unresolved(raw)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(raw)
  end
```

## `sym:cargo sealmap_model . sym/SymbolId#extend_path().`
`pub fn extend_path(&self, segment: &str) -> SymbolId` · L599-L613
> Continue an id with a segment of unknown kind: the result is a path id made of [`names`](Self::names) plus `segment`.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: names()
```

## `sym:cargo sealmap_model . sym/SymbolId#view().`
`pub fn view(&self) -> IdView<'_>` · L615-L661
> A borrowed view of the id's parts.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  participant sealmap_model__sym as sym mod
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: shape()
  alt Shape::Global { manager, name, version, descriptors }
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: unfield(name)
    opt via then
      sealmap_model__sym___tSymbolId->>sealmap_model__sym: unfield(version)
    end
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_descriptors(descriptors)
    opt via map
      sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(d.name)
    end
  else Shape::Path(body)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: scan_path(body)
  else Shape::Unresolved(raw)
    sealmap_model__sym___tSymbolId->>sealmap_model__sym: unquote(raw)
  end
```

## `sym:cargo sealmap_model . sym/SymbolId#display_path().`
`pub fn display_path(&self) -> String` · L663-L667
> The names joined by `::`, for human-facing labels.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tSymbolId as SymbolId
  sealmap_model__sym___tSymbolId->>sealmap_model__sym___tSymbolId: names()
```

## `sym:cargo sealmap_model . sym/print().`
`fn print(repr: &Repr) -> String` · L702-L738
> The canonical text of `repr`.
```mermaid
sequenceDiagram
  participant sealmap_model__sym as sym mod
  alt Repr::Global { package, descriptors }
    sealmap_model__sym->>sealmap_model__sym: write_field(&package.name, &out)
    opt Version::Release(v)
      sealmap_model__sym->>sealmap_model__sym: write_field(v, &out)
    end
  else Repr::Path(segs)
    loop for (i, s) in segs.iter().enumerate()
      sealmap_model__sym->>sealmap_model__sym: write_name(s, &out)
    end
  else Repr::Unresolved(n)
    sealmap_model__sym->>sealmap_model__sym: write_name(n, &out)
  end
```

## `sym:cargo sealmap_model . sym/raw_name_len().`
`fn raw_name_len(s: &str) -> usize` · L862-L880
> The length of the raw name at the start of `s` (quoted or simple).
```mermaid
sequenceDiagram
  participant sealmap_model__sym as sym mod
  opt b.first() == Some(&b'#96;')
    loop while i<b.len()
      opt b [i] == b'#96;'
        Note over sealmap_model__sym: return i + 1
      end
    end
    Note over sealmap_model__sym: return b.len()
  end
  loop each via find
    sealmap_model__sym->>sealmap_model__sym: is_simple(c)
  end
```

## `sym:cargo sealmap_model . sym/scan_descriptors().`
`fn scan_descriptors(s: &str) -> Vec<DescSpan<'_>>` · L882-L908
```mermaid
sequenceDiagram
  participant sealmap_model__sym as sym mod
  loop while pos<s.len()
    alt open @ (b'['| b'(')
      sealmap_model__sym->>sealmap_model__sym: raw_name_len(&_)
    else _
      sealmap_model__sym->>sealmap_model__sym: raw_name_len(rest)
    end
  end
```

## `sym:cargo sealmap_model . sym/scan_path().`
`fn scan_path(s: &str) -> Vec<&str>` · L910-L923
```mermaid
sequenceDiagram
  participant sealmap_model__sym as sym mod
  loop loop
    sealmap_model__sym->>sealmap_model__sym: raw_name_len(&_)
    opt not s [pos..].starts_with(#quot;::#quot;)
      Note over sealmap_model__sym: return out
    end
  end
```

## `sym:cargo sealmap_model . sym/Parser#peek().`
`fn peek(&self) -> Option<char>` · L955-L957
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: rest()
```

## `sym:cargo sealmap_model . sym/Parser#eat().`
`fn eat(&mut self, lit: &str) -> bool` · L959-L966
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: rest()
```

## `sym:cargo sealmap_model . sym/Parser#expect().`
`fn expect(&mut self, lit: &str, why: &'static str) -> Result<(), IdError>` · L968-L970
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(lit)
  opt not self.eat(lit)
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(why)
  end
```

## `sym:cargo sealmap_model . sym/Parser#done().`
`fn done(&self) -> Result<(), IdError>` · L972-L974
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  opt not self.pos == self.s.len()
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(#quot;unexpected t…)
  end
```

## `sym:cargo sealmap_model . sym/Parser#id().`
`fn id(mut self) -> Result<Repr, IdError>` · L976-L1011
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  participant sealmap_model__sym___tPackage as Package
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(SYM_PREFIX)
  opt !self.eat(SYM_PREFIX)
    Note over sealmap_model__sym___tParser: return Err(IdError::MissingPrefix)
  end
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(#quot;? #quot;)
  opt self.eat(#quot;? #quot;)
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: name()?
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: done()?
    Note over sealmap_model__sym___tParser: return Ok(Repr::Unresolved(name))
  end
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(#quot;extern #quot;)
  opt self.eat(#quot;extern #quot;)
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: name()?
    loop while self.eat(#quot;::#quot;)
      sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(#quot;::#quot;)
      sealmap_model__sym___tParser->>sealmap_model__sym___tParser: name()?
    end
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: done()?
    Note over sealmap_model__sym___tParser: return Ok(Repr::Path(segs))
  end
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: field()?
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: expect(#quot; #quot;, #quot;expected a s…)?
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: field()?
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: expect(#quot; #quot;, #quot;expected a s…)?
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: field()?
  sealmap_model__sym___tParser->>sealmap_model__sym___tPackage: Package::new(manager, name, version)?
  opt self.pos<self.s.len()
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: expect(#quot; #quot;, #quot;expected a s…)?
    opt self.pos == self.s.len()
      sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(#quot;expected a d…)
      Note over sealmap_model__sym___tParser: return Err(self.err(#quot;expected a descriptor#quot;))
    end
    loop while self.pos<self.s.len()
      sealmap_model__sym___tParser->>sealmap_model__sym___tParser: descriptor()?
    end
  end
```

## `sym:cargo sealmap_model . sym/Parser#field().`
`fn field(&mut self) -> Result<String, IdError>` · L1013-L1036
> A package field, up to (not including) the next single space or the end.
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  loop loop
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: rest()
  end
  opt out.is_empty()
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(#quot;empty packag…)
    Note over sealmap_model__sym___tParser: return Err(self.err(#quot;empty package field#quot;))
  end
```

## `sym:cargo sealmap_model . sym/Parser#name().`
`fn name(&mut self) -> Result<String, IdError>` · L1038-L1071
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(#quot;#96;#quot;)
  opt self.eat(#quot;#96;#quot;)
    loop loop
      sealmap_model__sym___tParser->>sealmap_model__sym___tParser: peek()
      alt None
        sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(#quot;unterminated…)
        Note over sealmap_model__sym___tParser: return Err(self.err(#quot;unterminated quoted name#quot;))
      else Some('#96;')
        sealmap_model__sym___tParser->>sealmap_model__sym___tParser: rest()
        opt not self.rest().starts_with(#quot;#96;#96;#quot;)
          opt !out.is_empty() && out.chars().all(is_simple)
            sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(#quot;a simple nam…)
            Note over sealmap_model__sym___tParser: return Err(self.err(#quot;a simple name must not be quoted#quot;))
          end
          Note over sealmap_model__sym___tParser: return Ok(out)
        end
      end
    end
  end
  loop while self.peek().is_some_and(is_simple)
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: peek()
  end
  opt self.pos == start
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(#quot;expected a n…)
    Note over sealmap_model__sym___tParser: return Err(self.err(#quot;expected a name#quot;))
  end
```

## `sym:cargo sealmap_model . sym/Parser#descriptor().`
`fn descriptor(&mut self) -> Result<Descriptor, IdError>` · L1073-L1106
```mermaid
sequenceDiagram
  participant sealmap_model__sym___tParser as Parser
  participant sealmap_model__sym___tDescriptor as Descriptor
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(#quot;[#quot;)
  opt self.eat(#quot;[#quot;)
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: name()?
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: expect(#quot;]#quot;, #quot;expected #96;]#96;#quot;)?
    sealmap_model__sym___tParser->>sealmap_model__sym___tDescriptor: Descriptor::type_parameter(name)
    Note over sealmap_model__sym___tParser: return Ok(Descriptor::type_parameter(name))
  end
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: eat(#quot;(#quot;)
  opt self.eat(#quot;(#quot;)
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: name()?
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: expect(#quot;)#quot;, #quot;expected #96;)#96;#quot;)?
    sealmap_model__sym___tParser->>sealmap_model__sym___tDescriptor: Descriptor::parameter(name)
    Note over sealmap_model__sym___tParser: return Ok(Descriptor::parameter(name))
  end
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: name()?
  sealmap_model__sym___tParser->>sealmap_model__sym___tParser: peek()
  alt Some('(')
    loop while self.peek().is_some_and(is_simple)
      sealmap_model__sym___tParser->>sealmap_model__sym___tParser: peek()
    end
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: expect(#quot;).#quot;, #quot;expected #96;).…)?
    sealmap_model__sym___tParser->>sealmap_model__sym___tDescriptor: Descriptor::new(name, _)
    Note over sealmap_model__sym___tParser: return Descriptor::new(name, Suffix::Method { disambigu…
  else _
    sealmap_model__sym___tParser->>sealmap_model__sym___tParser: err(#quot;expected a d…)
    Note over sealmap_model__sym___tParser: return Err(self.err(#quot;expected a descriptor suffix#quot;))
  end
```
