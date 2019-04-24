********************************************
Grid Schema Transaction Family Specification
********************************************

Overview
========

Grid Schema for Hyperledger Grid provides a reusable, standard approach to
defining, storing, and consuming properties within smart contracts, software
libraries, and network-based APIs.

Several components within Grid store and retrieve properties which are
defined at runtime. To properly store and validate these properties, we need
property definitions which minimally include the property’s type (integer,
string, enum, etc.). In addition, the properties (for example, product
description, GPS location, or product dimensions) should always be stored and
exchanged using the same format within Grid components.

State
=====
All Grid Schema objects are serialized using protocol buffers (protobufs) before
being stored in state. Theses objects include Schema, PropertyDefinition and
PropertyValues. Schemas are stored in a list to handle hash collisions.

.. note:: Organization and Agents in the Pike Transaction Family are used
    to enforce permissions on who is allowed to update a Schema.

PropertyDefinition
------------------

A property is defined using a ``PropertyDefinition`` which includes the
following:

- Data type (one of: BYTES, BOOLEAN, NUMBER, STRING, ENUM, STRUCT)
- Name
- Type description
- Optionality (whether or not the field is required)

.. code-block:: protobuf

  message PropertyDefinition {
      enum DataType {
          UNSET_DATA_TYPE = 0;
          BYTES = 1;
          BOOLEAN = 2;
          NUMBER = 3;
          STRING = 4;
          ENUM = 5;
          STRUCT = 6;
      }

      // The name of the property
      string name = 1;
      // The data type of the value; must not be set to UNSET_DATA_TYPE.
      DataType data_type = 2;
      // Indicates that this is a required property in the Schema
      bool required = 3;
      // An optional description of the field.
      string description = 4;

      // The exponent for a NUMBER property
      sint32 number_exponent = 10;
      // The list of values for an ENUM property; must not be empty/ for
      // properties of that type.
      repeated string enum_options = 11;
      // The list of property definitions for a STRUCT property; must  not be
      // empty for properties of that type.
      repeated PropertyDefinition struct_properties = 12;
  }

PropertyValue
-------------

A property value is defined using a ``PropertyValue`` which includes the
following:

- Data type (one of: BYTES, BOOLEAN, NUMBER, STRING, ENUM, STRUCT)
- Name
- Corresponding value of data type

.. code-block:: protobuf

  message PropertyValue {
      // The name of the property value.  Used to validate the property against
      // a Schema.
      string name = 1;
      // The data type of the property.  Indicates which value field the actual
      // value may be found.  Must not be set to ``UNSET_DATA_TYPE``.
      PropertyDefinition.DataType data_type = 2;

      // The value fields for the possible data types.  Only one of these will
      // contain a value, determined by the value of ``data_type``
      bytes bytes_value = 10;
      bool boolean_value = 11;
      sint64 number_value = 12;
      string string_value = 13;
      uint32 enum_value = 14;
      repeated PropertyValue struct_values = 15;
  }

Data Types
----------

Bytes
  A Bytes data type is an array of raw bytes.  This can be used to store
  arbitrary, opaque data. For example, a property with the Bytes data type could
  be used to store serialized JSON objects containing application metadata for a
  field, such as an image URL or style name.

  A bytes value is be represented as follows:

  .. code-block:: python

    PropertyDefinition(
        name="user_data",
        data_type=PropertyDefinition.DataType.Bytes,
        description="Arbitrary serialized user data."
    )

  Because this is a protobuf message, the default value for this field is an
  empty byte array.

Booleans
  A boolean data type restricts a value to True and False. Though boolean types
  could be stored in other integer (or byte) types using 0 or 1, an explicit
  boolean type assists in capturing intent and restricting the value.

  A boolean value is represented as follows:

  .. code-block:: python

    PropertyDefinition(
        name="is_enabled",
        data_type=PropertyDefinition.DataType.BOOLEAN,
        required=True,
        description="Indicates that the containing struct is enabled."
    )

  The value is represented as:

  .. code-block:: python

    PropertyValue(
        name="is_enabled",
        data_type=PropertyDefinition.DataType.BOOLEAN,
        boolean_value=True
    )

  Because this is a protobuf message, the default value for this field is
  ``False``.

Strings
  A string data type contains a standard UTF-8 encoded string value.

  A UTF-8 string value is represented as follows:

  .. code-block:: python

    PropertyDefinition(
        name="title",
        data_type=PropertyDefinition.DataType.STRING,
        required=True,
        description="A blog post title."
    )


  The value is represented as:

  .. code-block:: python

    PropertyValue(
        name="title"
        data_type=PropertyDefinition.DataType.STRING,
        string_value="My Very Nice Blog Example"
    )

  Because this is a protobuf message, the default value for this field is the
  empty string.

Numbers
  Numbers are represented as an integer with a given precision.  This can be
  thought of as akin to scientific notation. An instance of a number with this
  property definition is represented as a value (the significand) with the
  exponent (the order of magnitude) defined in the schema itself. So for
  example:

  ``(value: 24, exponent: 3)  -> 24 * 10^3  -> 24000``
  ``(value: 24, exponent: -3) -> 24 * 10^-3 -> 0.024``
  ``(value: 24, exponent: 0)  -> 24 * 10^0  -> 24``

  Importantly, this exponent is set on a Property's schema, not when the
  value is actually input. It affects the semantic meaning of integers
  stored under a Property, not any of the actual operations done with them.
  Properties with an exponent of 3 or -3 are always expressed as a whole
  integer of thousands or thousandths. For this reason, the exponent should be
  thought of more as a unit of measure than as true scientific notation.

  Standard integers are represented with the exponent set to zero.

  An integer value is represented as the following type:

  .. code-block:: python

    PropertyDefinition(
        name="quantity",
        data_type=PropertyDefinition.DataType.NUMBER,
        number_exponent=0,
        required=True,
        description="The count of values in this container"
    )

  This example shows an instance of a quantity of 23:

  .. code-block:: python

    PropertyValue(
        name="quantity",
        data_type=PropertyDefinition.DataType.NUMBER,
        number_value=23,
    )

  A fractional value is represented as the following type:

  .. code-block:: python

    PropertyDefinition(
        name="price",
        data_type=PropertyDefinition.DataType.NUMBER,
        number_exponent=-2,
        required=True,
        description="The the price this object"
    )

  This example shows an instance of a price with the value $0.23:

  .. code-block:: python

    PropertyValue(
        name="price",
        data_type=PropertyDefinition.DataType.NUMBER,
        number_value=23,
    )

  Because this is a protobuf message, the default exponent is ``0`` when the
  schema is created. Likewise, the default value for this property instance is
  ``0``.

Enums
  An enum data type restricts values to a limited set of possible values. The
  definition for this data type includes a list of string names describing a
  possible state of the enum. A ``PropertyValue`` for this data type contains
  an integer value corresponding to the index of a value in the ``enum_option``
  list.

  An enum value is represented as:

  .. code-block:: python

    PropertyDefinition(
        name='color',
        data_type=PropertyDefinition.DataType.ENUM,
        enum_options=['white', 'red', 'green', 'blue', 'blacklight'],
        required=True
    )

  An instance of a red enum is as follows:

  .. code-block:: python

    PropertyValue(
        name='color',
        data_type=PropertyDefinition.DataType.ENUM,
        enum_value=1
    )

  Due to the use of protobuf, the default value for ``enum_value`` is
  ``0``.

Structs
  A struct is a recursively defined collection of other named properties that
  represents two or more intrinsically linked values, like X/Y coordinates or
  RGB colors. These values can be of any Grid Schema data type, including
  STRUCT, which allows nesting to an arbitrary depth. Although versatile and
  powerful, structs are heavyweight and should be used conservatively;
  restrict struct use to linking values that must always be updated together.
  The transaction processor enforces this usage, rejecting any transactions
  that do not have a value for every property in a struct.

  Note that although structs are built using a list of PropertyDefinitions, any
  nested use of the required property is meaningless and is rejected by the
  transaction processor. As Properties are set in their entirety, either all of
  the struct is required or none of it is. In other words, partial structs are
  not allowed.

  A struct value is represented as follows:

  .. code-block:: python

    PropertyDefinition(
        name='shock',
        data_type=PropertyDefinition.DataType.STRUCT,
        struct_properties=[
            PropertyDefinition(
                name='speed',
                data_type=PropertyDefinition.DataType.NUMBER,
                number_exponent=-6),
            PropertyDefinition(
                name='duration',
                data_type=PropertyDefinition.DataType.NUMBER,
                number_exponent=-6),
        ],
        required=True
    )

  An instance of the ``shock`` struct is as follows:

  .. code-block:: python

    PropertyValue(
        name='shock',
        data_type=PropertyDefinition.DataType.STRUCT,
        struct_values=[
            PropertyValue(
                name='speed',
                data_type=PropertySchema.DataType.NUMBER,
                number_value=500000),
            PropertyValue(
                name='duration',
                data_type=PropertySchema.DataType.NUMBER,
                number_value=10000)
            ])

  The property value for a struct must contain all the struct values from the
  property definition, or it is invalid.  The defaults for the struct values
  themselves depend on their data types and/or the smart-contract implementer
  validation rules.

Schemas
-------

Property definitions are collected into a Schema data type, which defines all
the possible properties for an item that belongs to a given schema. Schemas
include the following:

- a name
- a description
- an owner
- a list of ``PropertyDefinitions``

.. code-block:: protobuf

  message Schema {
      // The name of the Schema.  This is also the unique identifier for the
      // Schema.
      string name = 1;
      // An optional description of the schema.
      string description = 2;
      // The Pike organization that has rights to modify the schema.
      string owner = 3;

      // The property definitions that make up the Schema; must not be empty.
      repeated PropertyDefinition properties = 10;
  }

An owner is an Organization Id that correlates to an Organization stored with
the Pike Transaction Family.

When the same address is computed for different schema, a collision occurs; all
colliding schemas are stored at the address in a SchemaList.

.. code-block:: protobuf

  // A SchemaList is used to mitigate hash collisions.
  message SchemaList {
      repeated Schema schemas = 1;
  }

A complete object representation can be built from the property definition
messages, and instances can be represented by constructing items with the
property value messages.

Suppose there is a requirement to store different types of light bulbs as part
of an application. A lightbulb may consist of the properties size, bulb type,
energy rating, and color.

We can define a Lightbulb schema as follows:

.. code-block:: python

  Schema(
      name="Lightbulb",
      description="Example Lightbulb schema",
      owner = "philips001"
      properties=[
          PropertyDefinition(
              name="size",
              data_type=PropertyDefinition.DataType.NUMBER,
              description="Lightbulb radius, in millimeters",
              number_exponent=0,
              required=True
          ),
          PropertyDefinition(
              name="bulb_type",
              data_type=PropertyDefinition.DataType.ENUM,
              enum_options=["filament", "CF", "LED"],
              required=True
          ),
          PropertyDefinition(
              name="energy_rating",
              data_type=PropertyDefinition.DataType.NUMBER,
              description="EnergyStar energy rating",
              number_exponent=0,
          )
          PropertyDefinition(
              name="color",
              data_type=PropertyDefinition.DataType.STRUCT,
              description="A named RGB Color value",
              struct_properties=[
                  PropertyDefinition(
                      name='name',
                      data_type=PropertyDefinition.DataType.STRING,
                  ),
                  PropertyDefinition(
                      name='rgb_hex',
                      data_type=PropertyDefinition.DataType.STRING,
                  )])])

Note: This example looks very similar to defining a struct property, but the
fields in a schema may be optional.

We can define a data structure that uses this schema for validation as follows:

.. code-block:: protobuf

  message Lightbulb {
      string id = 1;
      string production_org = 2;
      repeated PropertyValues properties = 3;
  }

A Lightbulb smart contract is responsible for validating the properties
against the Lightbulb schema.

Addressing
----------

Grid Schemas are stored under the Grid namespace ``621dee``. For each schemas,
the address is formed by concatenating the namespace, the special policy
namespace of ``01``, and the first 62 characters of the SHA-256 hash of the
schema name.

For example, the address of the ``Lightbulb`` schema defined in the example
above is (in Python):

.. code-block:: python

 "621dee" + "01" + hashlib.sha512("Lightbulb").encode("utf-8")).hexdigest()[:62]

To avoid hash collisions, schemas must be stored in a ``SchemaList``.

Transaction Payload and Execution
=================================

The following transactions and their execution rules are designed for the
Hyperledger Sawtooth platform and may differ for other transaction execution
platforms.

The header for the transactions includes the following:

- ``family_name``: ``"grid_schema"``
- ``family_version``: ``"1.0"``
- ``namespaces``: ``[ "621dee" ]``

SchemaPayload
-------------

SchemaPayload contains an action enum and the associated action payload.  This
allows for the action payload to be dispatched to the appropriate logic.

Only the defined actions are available and only one action payload should be
defined in the SchemaPayload.

.. code-block:: protobuf

  message SchemaPayload {
      enum Actions {
          UNSET_ACTION = 0;
          SCHEMA_CREATE = 1;
          SCHEMA_UPDATE = 2;
      }

      Action action = 1;

      SchemaCreateAction schema_create = 2;
      SchemaUpdateAction schema_update = 3
  }

SchemaCreateAction
------------------

SchemaCreateAction adds a new Schema to state.

.. code-block:: protobuf

  message SchemaCreateAction {
      string schema_name = 1;
      string description = 2;
      repeated PropertyDefinition properties = 10;
  }

The action is validated according to the following rules:

- If a Schema already exists with this name or the name is an empty string, the
  transaction is invalid.
- If the property list is empty, the transaction is invalid.
- The signer of the transaction must be an agent in Pike state and must belong
  to an organization in Pike state, otherwise the transaction is invalid.
- The agent must have the permission ``can_create_schema`` for the organization,
  otherwise the transaction is invalid.

The schema is created with the provided fields, in addition to the Pike
organization ID as the ``owner_id``. The schema is then stored in state.

The inputs for SchemaCreateAction must include:

- Address of the Agent submitting the transaction
- Address of the Schema

The outputs for SchemaCreateAction must include:

- Address of the Schema

SchemaUpdateAction
------------------

SchemaUpdateAction updates a Schema to state. This update only adds new
Properties to the Schema.

.. code-block:: protobuf

  message SchemaUpdateAction {
      string schema_name = 1;
      repeated PropertyDefinition properties = 2;
  }


The action is validated according to the following rules:

- If a Schema does not exist, the transaction is invalid.
- If the property list is empty, the transaction is invalid.
- If one of the new properties has the same name as a property already defined
  in the schema, the  transaction is invalid.
- The signer of the transaction must be an agent in the Pike state and must
  belong to an organization in Pike state, otherwise the transaction is invalid.
- The signer of the transaction must belong to the same organization matching
  the ``owner`` of the schema, otherwise the transaction is invalid.
- The agent must have the permission ``can_update_schema`` for the organization,
  otherwise the transaction is invalid.

The inputs for SchemaUpdateAction must include:

- Address of the Agent submitting the transaction
- Address of the Schema

The outputs for SchemaCreateAction must include:

- Address of the Schema

.. Licensed under Creative Commons Attribution 4.0 International License
.. https://creativecommons.org/licenses/by/4.0/
