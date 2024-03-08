<div align="center">
  <a href="https://github.com/ipvm-wg/homestar" target="_blank">
    <img src="https://raw.githubusercontent.com/ipvm-wg/homestar/main/assets/mascot_full_transparent.png" alt="Homestar logo" width="400"></img>
  </a>

  <h1 align="center">homestar-wasm</h1>

  <p>
    <a href="https://crates.io/crates/homestar-wasm">
      <img src="https://img.shields.io/crates/v/homestar-wasm?label=crates" alt="Crate">
    </a>
    <a href="https://github.com/ipvm-wg/homestar/blob/main/homestar-wasm/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/homestar-wasm">
      <img src="https://img.shields.io/static/v1?label=Docs&message=wasm.docs.rs&color=pink" alt="Docs">
    </a>
    <a href="https://fission.codes/discord">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

##

## Outline

- [Description](#description)
- [Interpreting between IPLD and WIT](#interpreting-between-ipld-and-wit)

## Description

This *wasm* library manages the [wasmtime][wasmtime] runtime, provides the
[IPLD][ipld] to/from Wasm Interace Types ([WIT][wit])
interpreter/translation-layer, and implements the input interface for working
with Ipvm's standard Wasm tasks.

For more information, please go to our [Homestar Readme][homestar-readme].

## Interpreting between IPLD and WIT

Our recursive interpreter is able to bidirectionally translate between
the runtime [IPLD data model][ipld-data-model] and [WIT][wit] values, based on
known [WIT][wit] interface types.

### Primitive Types

We'll start by covering WIT [primitive types][wit-primitive].

#### Booleans

This section outlines the translation process between IPLD boolean values
(`Ipld::Bool`) and [WIT `bool` runtime values][wit-val].

- **IPLD to WIT Translation**:

  When a WIT function expects a `bool` input, an `Ipld::Bool` value (either
  `true` or `false`) is mapped to a `bool` WIT runtime
  value.

  **Example**: Consider a WIT function defined as follows:

  ```wit
  export fn: func(a: bool) -> bool;
  ```

  Given a JSON input for this function:

  ```json
  {
    "args": [true]
  }
  ```

  `true` is converted into an `Ipld::Bool`, which is then translated and
  passed into `fn` as a boolean argument (`bool`).

- **WIT to IPLD Translation**:

  Conversely, when a boolean value is returned from a WIT function, it can be
  translated back into an `Ipld::Bool`.

**IPLD Schema Definition**:

```ipldsch
type IPLDBooleanAsWit bool
```

#### Integers

This section outlines the translation process between IPLD integer values
(`Ipld::Integer`) and [WIT `integer` rutime values][wit-val].

The [Component Model][wit] supports these [integer][wit-integer] types:

```ebnf
ty ::= 'u8' | 'u16' | 'u32' | 'u64'
     | 's8' | 's16' | 's32' | 's64'
```

- **IPLD to WIT Translation**:

  Typically, when a WIT function expects an integer input, an `Ipld::Integer`
  value is mapped to an integer WIT runtime value.

  **Example**: Consider a WIT function defined as follows:

  ```wit
  export fn: func(a: s32) -> s32;
  ```

  Given a JSON input for this function:

  ```json
  {
    "args": [1]
  }
  ```

  `1` is converted into an `Ipld::Integer`, which is then translated and
  passed into `fn` as an integer argument (`s32`).

  **Note**: However, if the input argument to the WIT interface is a `float`
  type, but the incoming value is an `Ipld::Integer`, then the IPLD value will
  be cast to a `float`, and remain as one for the rest of the computation. The cast is
  to provide affordances for JavaScript where, for example, the number `1.0` is converted to `1`.

- **WIT to IPLD Translation**:

  Conversely, when an integer value (not a float) is returned from a WIT
  function, it can be translated back into an `Ipld::Integer`.

**IPLD Schema Definitions**:

```ipldschme
type IPLDIntegerAsWit union {
  | U8        int
  | U16       int
  | U32       int
  | U64       int
  | S8        int
  | S16       int
  | S32       int
  | S64       int
  | Float32In int
  | Float64In int
} representation kinded

type WitAsIpldInteger union {
  | U8          int
  | U16         int
  | U32         int
  | U64         int
  | S8          int
  | S16         int
  | S32         int
  | S64         int
  | Float32Out  float
  | Float64Out  float
} representation kinded
```

#### Floats

This section outlines the translation process between IPLD float values
(`Ipld::Float`) and [WIT `float` runtime values][wit-val].

The [Component Model][wit] supports these Float types:

```ebnf
ty ::= 'float32' | 'float64'
```

- **IPLD to WIT Translation**:

  When a WIT function expects a float input, an `Ipld::Float` value is
  mapped to a float WIT runtime value. Casting is done to convert from `f32` to
  `f64` if necessary.

  **Example**: Consider a WIT function defined as follows:

  ```wit
  export fn: func(a: f64) -> f64;
  ```

  Given a JSON input for this function:

  ```json
  {
    "args": [1.0]
  }
  ```

  `1.0` is converted into an `Ipld::Float`, which is then translated and
  passed into `fn` as a float argument (`f64`).

- **WIT to IPLD Translation**:

  Conversely, when a `float32` or `float64` value is returned from a WIT
  function, it can be translated back into an `Ipld::Float`.

  **Note**: In converting from `float32` to `float64`, the latter of which is
  the default precision for [IPLD][ipld-float], precision will be lost.
  **The interpreter will use decimal precision in this conversion**.

**IPLD Schema Definitions**:

```ipldsch
type IPLDFloatAsWit union {
  | Float32 float
  | Float64 float
} representation kinded

type WitAsIpldFloat union {
  | Float32 float
  | Float64 float
} representation kinded
```

#### Strings

This section outlines the translation process between IPLD string values
(`Ipld::String`) and various [WIT runtime values][wit-val]. A `Ipld::String` value can be
interpreted as one of a `string`, `char`, `list<u8>`, or an `enum` discriminant
(which has no payload).

- `string`

  * **IPLD to WIT Translation**

    When a WIT function expects a `string` input, an `Ipld::String` value is
    mapped to a `string` WIT runtime value.

    **Example**:

    ```wit
    export fn: func(a: string) -> string;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": ["Saspirilla"]
    }
    ```

    `"Saspirilla"` is converted into an `Ipld::String`, which is then translated
    and passed into `fn` as a string argument (`string`).

  * **WIT to IPLD Translation**:

    Conversely, when a `string` value is returned from a WIT function, it is
    translated back into an `Ipld::String`.

- `char`

  * **IPLD to WIT Translation**

    When a WIT function expects a `char` input, an `Ipld::String` value is
    mapped to a `char` WIT runtime value.

    **Example**:

    ```wit
    export fn: func(a: char) -> char;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": ["S"]
    }
    ```

    `"S"`is converted into an `Ipld::String`, which is then translated and
    passed into `fn` as a char argument (`char`).

  * **WIT to IPLD Translation**:

    Conversely, when a char value is returned from a WIT function, it is
    translated back into an `Ipld::String`.

- `list<u8>`

  * **IPLD to WIT Translation**

    When a WIT function expects a `list<u8>` input, an `Ipld::String` value is
    mapped to a `list<u8>` WIT runtime value.

    **Example**:

    ```wit
    export fn: func(a: list<u8>) -> list<u8>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": ["aGVsbDA"]
    }
    ```

    `"aGVsbDA"` is converted into an `Ipld::String`, which is then translated
    into bytes and passed into `fn` as a `list<u8>` argument.

  * **WIT to IPLD Translation**:

    **Here, when a `list<u8>` value is returned from a WIT function, it is
    translated into an `Ipld::Bytes` value, which is the proper type**.

- [`enum`][wit-enum]:

  An enum statement defines a new type which is semantically equivalent to a
  variant where none of the cases have a payload type.

  * **IPLD to WIT Translation**

    When a WIT function expects an `enum` input, an `Ipld::String` value is
    mapped to a `enum` WIT runtime value.

    **Example**:

    ```wit
    enum color {
        Red,
        Green,
        Blue
    }

    export fn: func(a: color) -> string;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": ["Green"]
    }
    ```

    `"Green"` is converted into an `Ipld::String`, which is then translated and
    passed into `fn` as a enum argument (`color`). **You'll have to provide a
    string that matches on one of the discriminants**.

  * **WIT to IPLD Translation**:

    Conversely, when an enum value is returned from a WIT function, it can be
    translated back into an `Ipld::String` value.

**IPLD Schema Definitions**:

``` ipldsch
type Enum enum {
  | Red
  | Green
  | Blue
}

type IPLDStringAsWit union {
 | Enum     Enum
 | String   string
 | Char     string
 | Listu8In string
} representation kinded

type WitAsIpldString union {
 | Enum      Enum
 | String    string
 | Char      string
 | Listu8Out bytes
} representation kinded
```

#### Bytes

This section outlines the translation process between IPLD bytes values
(`Ipld::Bytes`) and various [WIT runtime values][wit-val]. A `Ipld::Bytes` value
can be interpreted either as a `list<u8>` or `string`.

- [`list`][wit-list]:

  * **IPLD to WIT Translation**

    When a WIT function expects a `list<u8>` input, an `Ipld::Bytes` value is
    mapped to a `list<u8>` WIT runtime value.

    **Example**:

    ```wit
    export fn: func(a: list<u8>) -> list<u8>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [{"/": {"bytes": "aGVsbDA"}}]
    }
    ```

    `"aGVsbDA"` is converted into an `Ipld::Bytes`, which is then translated
    into bytes and passed into `fn` as a `list<u8>` argument.

  * **WIT to IPLD Translation**:

    Conversely, when a `list<u8>` value is returned from a WIT function, it is
    translated back into an `Ipld::Bytes` value if the list contains valid
    `u8` values.

- `string`

  * **IPLD to WIT Translation**

    When a WIT function expects a `string` input, an `Ipld::Bytes` value is
    mapped to a `string` WIT runtime value.

    **Example**:

    ```wit
    export fn: func(a: string) -> string;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [{"/": {"bytes": "aGVsbDA"}}]
    }
    ```

    `"aGVsbDA"` is converted into an `Ipld::Bytes`, which is then translated
    into a `string` and passed into `fn` as a `string` argument.

  * **WIT to IPLD Translation**:

    **Here, when a string value is returned from a WIT function, it is
    translated into an `Ipld::String` value, because we can't determine if it
    was originally `bytes`**.

**IPLD Schema Definitions**:

``` ipldsch
type IPLDBytesAsWit union {
  | ListU8   bytes
  | StringIn bytes
} representation kinded

type WitAsIpldBytes union {
  | ListU8    bytes
  | StringOut string
} representation kinded
```


#### Nulls

This section outlines the translation process between IPLD null values
(`Ipld::Null`) and various [WIT runtime values][wit-val]. A `Ipld::Null` value
can be interpreted either as a `string` or `option`.

**We'll cover only the `string` case here** and return to the `option` case
below.

* **IPLD to WIT Translation**

  When a WIT function expects a `string` input, an `Ipld::Null` value is
  mapped as a `"null"` `string` WIT runtime value.

  **Example**:

  ```wit
  export fn: func(a: string) -> string;
  ```

  Given a JSON input for this function:

  ```json
  {
    "args": [null]
  }
  ```

  `null` is converted into an `Ipld::Null`, which is then translated and
  passed into `fn` as a `string` argument with the value of `"null"`.

* **WIT to IPLD Translation**:

  Conversely, when a `string` value of `"null"` is returned from a WIT function,
  it can be translated into an `Ipld::Null` value.

**IPLD Schema Definitions**:

``` ipldsch
type None unit representation null

type IPLDNullAsWit union {
  | None
  | String string
} representation kinded

type WitAsIpldNull union {
  | None
  | String string
} representation kinded
```

#### Links

This section outlines the translation process between IPLD link values
(`Ipld::Link`) and [WIT `string` runtime values][wit-val]. A `Ipld::Link` is always
interpreted as a `string` in WIT, and vice versa.

* **IPLD to WIT Translation**

  When a WIT function expects a `string` input, an `Ipld::Link` value is
  mapped to a `string` WIT runtime value, translated accordingly based
  on the link being [Cidv0][cidv0] or [Cidv1][cidv1].

  **Example**:

  ```wit
  export fn: func(a: string) -> string;
  ```

  Given a JSON input for this function:

  ```json
  {
    "args": ["bafybeia32q3oy6u47x624rmsmgrrlpn7ulruissmz5z2ap6alv7goe7h3q"]
  }
  ```

  `"bafybeia32q3oy6u47x624rmsmgrrlpn7ulruissmz5z2ap6alv7goe7h3q"` is converted
  into an `Ipld::Link`, which is then translated and passed into `fn` as a
  `string` argument.

* **WIT to IPLD Translation**:

  Conversely, when a `string` value is returned from a WIT function, and if it
  can be converted to a Cid, it can then be translated into an `Ipld::Link`
  value.

**IPLD Schema Definitions**:

``` ipldsch
type IPLDLinkAsWit &String link

type WitAsIpldLink &String link
```

### Non-primitive Types

Next, we'll cover the more interesting, WIT non-primitive types.

#### List Values

This section outlines the translation process between IPLD list values
(`Ipld::List`) and various [WIT runtime values][wit-val]. A `Ipld::List`
value can be interpreted as one of a `list`, `tuple`, set of `flags`,
or a `result`.

**We'll return to the `result` case below, and cover the rest of the
possibilities here**.

- [`list`][wit-list]

  * **IPLD to WIT Translation**

    When a WIT function expects a `list` input, an `Ipld::List` value is
    mapped to a `list` WIT runtime value.

    **Example**:

    ```wit
    export fn: func(a: list<s32>, b: s32) -> list<s32>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [[1, 2, 3], 44]
    }
    ```

    `[1, 2, 3]` is converted into an `Ipld::List`, which is then translated
    and passed into `fn` as a `list<s32>` argument.

  * **WIT to IPLD Translation**:

    Conversely, when a `list` value is returned from a WIT function, it is
    translated back into an `Ipld::List` value.

- [`tuple`][wit-tuple]:

  * **IPLD to WIT Translation**

    When a WIT function expects a `tuple` input, an `Ipld::List` value is
    mapped to a `tuple` WIT runtime value.

    **Example**:

    ```wit
    type ipv6-socket-address = tuple<u16, u16, u16, u16, u16, u16, u16, u16>;

    export fn: func(a: ipv6-socket-address) -> tuple<u32, u32>;
    ```

    Given an JSON input for this function:

    ```json
    {
      "args": [[8193, 3512, 34211, 0, 0, 35374, 880, 29492]]
    }
    ```

    `[8193, 3512, 34211, 0, 0, 35374, 880, 29492]` is converted into an
    `Ipld::List`, which is then translated and passed into `fn` as a
    `tuple<u16, u16, u16, u16, u16, u16, u16, u16>` argument.

    **If the length of list does not match not match the number of fields in the
    tuple interface type, then an error will be thrown in the interpreter.**

  * **WIT to IPLD Translation**:

    Conversely, when a `tuple` value is returned from a WIT function, it is
    translated back into an `Ipld::List` value.

- [`flags`][wit-flags]:

  `flags` represent a bitset structure with a name for each bit. The type
  represents a set of named booleans. In an instance of the named type, each flag will
  be either true or false.

  * **IPLD to WIT Translation**

    When a WIT function expects a `flags` input, an `Ipld::List` value is
    mapped to a `flags` WIT runtime value.

    When used as an input, you can set the flags you want turned on/true as an
    inclusive subset of strings. When used as an output, you will get a list of
    strings representing the flags that are set to true.


    **Example**:

    ```wit
    flags permissions {
        read,
        write,
        exec,
    }

    export fn: func(perm: permissions) -> bool;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [["read", "write"]]
    }
    ```

    `[read, write]` is converted into an `Ipld::List`, which is then translated
    and passed into `fn` as a `permissions` argument.

  * **WIT to IPLD Translation**:

    Conversely, when a `flags` value is returned from a WIT function, it is
    translated back into an `Ipld::List` value.


**IPLD Schema Definitions**:

``` ipldsch
type IPLDListAsWit union {
  | List [any]
  | Tuple [any]
  | Flags [string]
} representation kinded

type WitAsIpldList union {
  | List [any]
  | Tuple [any]
  | Flags [string]
} representation kinded
```

#### Maps

This section outlines the translation process between IPLD map values
(`Ipld::Map`) and various [WIT runtime values][wit-val]. A `Ipld::Map`
value can be interpreted as one of a `record`, `variant`, or
a `list` of two-element `tuples`.

- [`record`][wit-record]:

  * **IPLD to WIT Translation**

    When a WIT function expects a `record` input, an `Ipld::Map` value is
    mapped to a `record` WIT runtime value.

    **Example**:

    ```wit
    record pair {
        x: u32,
        y: u32,
    }

    export fn: func(a: pair) -> u32;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [{"x": 1, "y": 2}]
    }
    ```

    `{"x": 1, "y": 2}` is converted into an `Ipld::Map`, which is then
    translated and passed into `fn` as a `pair` argument.

    **The keys in the map must match the field names in the record type**.

  * **WIT to IPLD Translation**:

    Conversely, when a `record` value is returned from a WIT function, it is
    translated back into an `Ipld::Map` value.

- [`variant`][wit-variant]:

  A variant statement defines a new type where instances of the type match
  exactly one of the variants listed for the type. This is similar to a
  "sum" type in algebraic datatypes (or an enum in Rust if you're familiar
  with it). Variants can be thought of as tagged unions as well.

  Each case of a variant can have an optional type / payload associated with it
  which is present when values have that particular case's tag.

  * **IPLD to WIT Translation**

    When a WIT function expects a `variant` input, an `Ipld::Map` value is
    mapped to a `variant` WIT runtime value.

    **Example**:

    ```wit

    variant filter {
        all,
        none,
        some(list<string>),
    }

    export fn: func(a: filter);
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [{"some" : ["a", "b", "c"]}]
    }
    ```

    `{"some" : ["a", "b", "c"]}` is converted into an `Ipld::Map`, which is
    then translated and passed into `fn` as a `filter` argument, where the key
    is the variant name and the value is the payload.

    **The keys in the map must match the variant names in the variant type**.

  * **WIT to IPLD Translation**:

    Conversely, when a `variant` value is returned from a WIT function, it is
    translated back into an `Ipld::Map` value where the tag is the key and
    payload is the value.

- [`list`][wit-list]:

  * **IPLD to WIT Translation**

    When a WIT function expects a nested `list` of two-element `tuples` as input,
    an `Ipld::Map` value is mapped to that specific WIT runtime value.

    **Example**:

    ```wit
    export fn: func(a: list<tuple<string, u32>>) -> list<u32>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [{"a": 1, "b": 2}]
    }
    ```

    `{"a": 1, "b": 2}` is converted into an `Ipld::Map`, which is then
    translated and passed into `fn` as a `list<tuple<string, u32>>` argument.

  * **WIT to IPLD Translation**:

    Conversely, when a `list` of two-element `tuples` is returned from a WIT
    function, it can be translated back into an `Ipld::Map` value.

**IPLD Schema Definitions**:

``` ipldsch
type TupleAsMap {string:any} representation listpairs

type IPLDMapAsWit union {
  | Record {string:any}
  | Variant {string:any}
  | List TupleAsMap
} representation kinded

type WitAsIpldMap union {
  | Record {string:any}
  | Variant {string:any}
  | List TupleAsMap
} representation kinded
```

#### WIT Options

This section outlines the translation process between [WIT option runtime values][wit-val]
(of type `option`) and various IPLD values. An [`option`][wit-option] can be interpreted
as either a `Ipld::Null` or of any other IPLD value.

* **IPLD to WIT Translation**

  When a WIT function expects an `option` as input, an `Ipld::Null` value is
  mapped to the `None`/`Unit` case for a WIT option. Otherwise, any other IPLD
  value will be mapped to its matching WIT runtime value directly.

  **Example**:

  ```wit
  export fn: func(a: option<s32>) -> option<s32>;
  ```

  * `Some` case:

    - **Json Input**:

      ```json
      {
        "args": [1]
      }
      ```

  * `None` case:

    - **Json Input**:

      ```json
      {
        "args": [null]
      }
      ```

  `1` is converted into an `Ipld::Integer`, which is then translated and
  passed into `fn` as an integer argument (`s32`), as the `Some` case of the
  option.

  `null` is converted into an `Ipld::Null`, which is then translated and
  passed into `fn` as a `None`/`Unit` case of the option (i.e. no value in WIT).

  Essentially, you can view this as `Ipld::Any` being the `Some` case and
  `Ipld::Null` being the `None` case.

* **WIT to IPLD Translation**:

  Conversely, when an `option` value is returned from a WIT function, it can be
  translated back into an `Ipld::Null` value if it's the `None`/`Unit` case, or
  any other IPLD value if it's the `Some` case.

**IPLD Schema Definitions**:

``` ipldsch
type IpldAsWitOption union {
  | Some any
  | None
} representation kinded

type WitAsIpldOption union {
  | Some any
  | None
} representation kinded
```

#### WIT Results

This section outlines the translation process between [WIT result runtime values][wit-val]
(of type `result`) and various IPLD values. We treat result as Left/Right
[either][either] types over an `Ipld::List` of two elements.

A [`result`][wit-result] can be interpreted as one of these patterns:

- `Ok` (with a payload)

  * **IPLD to WIT Translation**

    When a WIT function expects a `result` as input, an `Ipld::List` value can
    be mapped to the `Ok` case of the `result` WIT runtime value, including
    a payload.

    **Example**:

    ```wit
    export fn: func(a: result<s32, string>) -> result<s32, string>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [[47, null]]
    }
    ```


    `[47, null]` is converted into an `Ipld::List`, which is then translated
    and passed into `fn` as an `Ok` case of the `result` argument with a
    payload of `47` matching the `s32` type on the left.

  * **WIT to IPLD Translation**:

    Conversely, when a `result` value is returned from a WIT function, it can
    be translated back into an `Ipld::List` of this specific structure.

- `Err` (with a payload)

  * **IPLD to WIT Translation**

    **Example**:

    ```wit
    export fn: func(a: result<s32, string>) -> result<s32, string>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [[null, "error message"]]
    }
    ```

    `[null, "error message"]` is converted into an `Ipld::List`, which is
    then translated and passed into `fn` as an `Err` case of the `result`
    argument with a payload of `"error message"` matching the `string` type
    on the right.

  * **WIT to IPLD Translation**:

      Conversely, when a `result` value is returned from a WIT function, it can
      be translated back into an `Ipld::List` of this specific structure.

- `Ok` case (without a payload)

  * **IPLD to WIT Translation**

    **Example**:

    ```wit
    export fn: func(a: result<_, string>) -> result<_, string>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [[47, null]]
    }
    ```

    `[47, null]` is converted into an `Ipld::List`, which is then translated
    and passed into `fn` as an `Ok` case of the `result` argument. The payload
    is ignored as it's not needed (expressed in the type as `_` above), so
    `47` is not used.

  * **WIT to IPLD Translation**:

    **Here, when this specific `Ok` case is returned from a WIT function, it can
    be translated back into an `Ipld::List`, but one structured as
    `[1, null]` internally, which signifies the `Ok` (not error) case, with
    the `1` payload discarded.**

- `Err` case (without a payload)

  * **IPLD to WIT Translation**

    **Example**:

    ```wit
    export fn: func(a: result<s32, _>) -> result<s32, _>;
    ```

    Given a JSON input for this function:

    ```json
    {
      "args": [[null, "error message"]]
    }
    ```

    `[null, "error message"]` is converted into an `Ipld::List`, which is
    then translated and passed into `fn` as an `Err` case of the `result`
    argument. The payload is ignored as it's not needed (expressed in the type
    as `_` above), so `"error message"` is not used.

  * **WIT to IPLD Translation**:

    **Here, when this specific `Err` case is returned from a WIT function, it
    can be translated back into an `Ipld::List`, but one structured as
    `[null, 1]` internally, which signifies the `Err` (error) case, with
    the `1` payload discarded.**

**IPLD Schema Definitions**:

``` ipldsch
type Null unit representation null

type IpldAsWitResult union {
  | Ok [any, Null]
  | Err [Null, any]
} representation kinded

type WitAsIpldResult union {
  | Ok [any, Null]
  | OkNone [1, Null]
  | Err [Null, any]
  | ErrNone [Null, 1]
} representation kinded
```

**Note**: `any` is used here to represent any type that's not `Null`. So,
given an input with a `result` type, the JSON value of

```json
{
  "args": [null, null]
}
```

will fail to be translated into a Wit `result`runtime value, as it's ambiguous
which case it should be mapped to.

[cidv0]: https://github.com/multiformats/cid?tab=readme-ov-file#cidv0
[cidv1]: https://github.com/multiformats/cid?tab=readme-ov-file#cidv1
[either]: https://www.scala-lang.org/api/2.13.6/scala/util/Either.html
[homestar-readme]: https://github.com/ipvm-wg/homestar/blob/main/README.md
[ipld]: https://ipld.io/
[ipld-data-model]: https://ipld.io/docs/data-model/
[ipld-float]: https://ipld.io/design/tricky-choices/numeric-domain/#floating-point
[ipld-type]: https://docs.rs/libipld/latest/libipld/ipld/enum.Ipld.html
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
[wit-primitive]: https://component-model.bytecodealliance.org/design/wit.html#primitive-types
[wit-enum]: https://component-model.bytecodealliance.org/design/wit.html#enums
[wit-flags]: https://component-model.bytecodealliance.org/design/wit.html#flags
[wit-integer]: https://component-model.bytecodealliance.org/design/wit.html#built-in-types
[wit-list]: https://component-model.bytecodealliance.org/design/wit.html#lists
[wit-option]: https://component-model.bytecodealliance.org/design/wit.html#options
[wit-record]: https://component-model.bytecodealliance.org/design/wit.html#records
[wit-result]: https://component-model.bytecodealliance.org/design/wit.html#results
[wit-tuple]: https://component-model.bytecodealliance.org/design/wit.html#tuples
[wit-val]: https://docs.wasmtime.dev/api/wasmtime/component/enum.Val.html
[wit-variant]: https://component-model.bytecodealliance.org/design/wit.html#variants
