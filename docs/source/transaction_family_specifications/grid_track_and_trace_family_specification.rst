*****************************************************
Grid Track and Trace Transaction Family Specification
*****************************************************

Overview
========

The Grid Track and Trace transaction family allows users to track
goods as they move through a supply chain. Records for goods include a
history of ownership and custodianship, as well as histories for a
variety of properties such as temperature and location. These properties are
managed using Grid Schemas.


State
=====

All Grid Track and Trace objects are serialized using Protocol Buffers before
being stored in state. These objects include: Records, Proposals, and
Properties (accompanied by their auxiliary PropertyPage objects). As described
in the Addressing_ section below, these objects are stored in separate
sub-namespaces under the Grid Track and Trace namespace. To handle hash
collisions, all objects are stored in lists within protobuf "List" objects.

.. note:: In addition to the messages defined in Grid Track and Trace, this
    transaction family also makes use of Agents (as defined in the :doc:`Pike
    Transaction Family<pike_transaction_family>`), as well as Schemas,
    PropertyDefinitions, and PropertyValues (as defined in the :doc:`Grid
    Schema Transaction Family<grid_schema_family_specification>`). Clients
    and contracts that implement this specification will need to orchestrate
    these transaction families together in order to create a working
    application.

Records
-------

Records represent the goods being tracked by Grid Track and Trace. Almost
every transaction references some Record.

A Record contains a unique identifier, the name of a Schema, and
lists containing the history of its owners and custodians. It also
contains a ``final`` flag indicating whether further updates can be
made to the Record and its Properties. If this flag is set to true,
then no further updates can be made to the Record, including changing
its ``final`` flag.

.. code-block:: protobuf

    message Record {
        message AssociatedAgent {
            // Agent's public key.
            string agent_id = 1;

            // The approximate time this agent was associated, as a Unix UTC timestamp.
            uint64 timestamp = 2;
        }

        // User-defined natural key which identifies the object in the real world
        // (for example a serial number).
        string record_id = 1;

        // Name of the Schema used by the record.
        string schema = 2;

        // Ordered oldest to newest by timestamp.
        repeated AssociatedAgent owners = 3;
        repeated AssociatedAgent custodians = 4;

        // Flag indicating whether the Record can be updated. If it is set
        // to true, then the record has been finalized and no further
        // changes can be made to it or its Properties.
        bool final = 5;
    }


Note that while information about a Record's owners and custodians are
included in the object, information about its Properties are stored
separately (see the Properties_ section below).

Records whose addresses collide are stored in a list, sorted by record ID.

.. code-block:: protobuf

    message RecordList {
        repeated Record entries = 1;
    }

.. _Properties:

Properties
----------

Historical data pertaining to a particular data field of a tracked
object are stored as Properties, represented as a list of values
accompanied by a timestamp and a reporter identifier.

The whole history of updates to Record data is stored in current state
because this allows for more flexibility in writing transaction rules.
For example, in a fish track-and-trade system, there might be a rule
that no fish can be exchanged whose temperature has gone above 40
degrees. This means, however, that it would be impractical to store
all of a Record's data at one address, since adding a single update
would require reading the entire history of each of the Record's
Properties out of state, adding the update, then writing it all back.

To solve this problem, Properties are stored in their own namespace
derived from their name and associated Record. Since some Properties
may have thousands of updates, four characters are reserved at the end
of that namespace in order to paginate a Property's history. The
Property itself (along with name, Record identifier, authorized
reporters, and paging information) is stored at the namespace ending
in ``0000``. The namespaces ending in ``0001`` to ``ffff`` will each
store a PropertyPage containing up to 256 reported values (which
include timestamps and their reporter's identity). Any Transaction
updating the value of a Property first reads out the PropertyList
object at ``0000`` and then reads out the appropriate
PropertyPageList before adding the update and writing the new
PropertyPageList back to state.

The Transaction Processor treats these pages as a ring buffer, so that
when page ``ffff`` is filled, the next update will erase the entries
at page ``0001`` and be stored there, and subsequent page-filling will
continue to overwrite the next oldest page. This ensures no Property
ever runs out of space for new updates. Under this scheme, 16^2 *
(16^4 - 1) = 16776960 entries can be stored before older updates are
overwritten.

Updates to Properties are in the format of PropertyValue (defined in the Grid
Schema Transaction Family). The type of update is indicated by a tag belonging
to the PropertyDefinition object. For more information about PropertyValues and
PropertyDefinitions, please see the :doc:`grid_schema_family_specification`.

.. code-block:: protobuf

    message Property {
        message Reporter {
            // The public key of the Agent authorized to report updates.
            string public_key = 1;

            // A flag indicating whether the reporter is authorized to send updates.
            // When a reporter is added, this is set to true, and a `RevokeReporter`
            // transaction sets it to false.
            bool authorized = 2;

            // An update must be stored with some way of identifying which
            // Agent sent it. Storing a full public key for each update would
            // be wasteful, so instead Reporters are identified by their index
            // in the `reporters` field.
            uint32 index = 3;
        }

        // The name of the Property, e.g. "temperature". This must be unique among
        // Properties.
        string name = 1;

        // The natural key of the Property's associated Record.
        string record_id = 2;

        // The name of the PropertyDefinition that defines this record.
        PropertyDefinition property_definition = 3;

        // The Reporters authorized to send updates, sorted by index. New
        // Reporters should be given an index equal to the number of
        // Reporters already authorized.
        repeated Reporter reporters = 4;

        // The page to which new updates are added. This number represents
        // the last 4 hex characters of the page's address. Consequently,
        // it should not exceed 16^4 = 65536.
        uint32 current_page = 5;

        // A flag indicating whether the first 16^4 pages have been filled.
        // This is used to calculate the last four hex characters of the
        // address of the page containing the earliest updates. When it is
        // false, the earliest page's address will end in "0001". When it is
        // true, the earliest page's address will be one more than the
        // current_page, or "0001" if the current_page is "ffff".
        bool wrapped = 6;
    }

    message PropertyPage {
        message ReportedValue {
            // The index of the reporter id in reporters field.
            uint32 reporter_index = 1;

            // The approximate time this value was reported, as a Unix UTC timestamp.
            uint64 timestamp = 2;

            PropertyValue value = 3;
        }

        // The name of the page's associated Property and the record_id of
        // its associated Record. These are required to distinguish pages
        // with colliding addresses.
        string name = 1;
        string record_id = 2;

        // ReportedValues are sorted first by timestamp, then by reporter_index.
        repeated ReportedValue reported_values = 3;
    }


Properties and PropertyPages whose addresses collide are stored in
lists alphabetized by Property name.

.. code-block:: protobuf

    message PropertyList {
        repeated Property entries = 1;
    }

    message PropertyPageList {
        repeated PropertyPage entries = 1;
    }

Proposals
---------

A Proposal is an offer from the owner or custodian of a Record to
authorize another Agent as an owner, custodian, or reporter for that
Record. Proposals are tagged as being for transfer of ownership,
transfer of custodianship, or authorization of a reporter for some
Properties. Proposals are also tagged as being open, accepted,
rejected, or canceled. There cannot be more than one open Proposal for
a specified role for each combination of Record, receiving Agent, and
issuing Agent.

.. code-block:: protobuf

    message Proposal {
        enum Role {
            OWNER = 0;
            CUSTODIAN = 1;
            REPORTER = 2;
        }

        enum Status {
            OPEN = 0;
            ACCEPTED = 1;
            REJECTED = 2;
            CANCELED = 3;
        }

        // The Record that this proposal applies to.
        string record_id = 1;

        // The approximate time this proposal was created, as a Unix UTC timestamp.
        uint64 timestamp = 2;

        // The public key of the Agent sending the Proposal. This Agent must
        // be the owner of the Record (or the custodian, if the Proposal is
        // to transfer custodianship).
        string issuing_agent = 3;

        // The public key of the Agent to whom the Proposal is sent.
        string receiving_agent = 4;

        // What the Proposal is for -- transferring ownership, transferring
        // custodianship, or authorizing a reporter.
        Role role = 5;

        // The names of properties for which the reporter is being authorized
        // (empty for owner or custodian transfers).
        repeated string properties = 6;

        // The status of the Proposal. For a given Record and receiving
        // Agent, there can be only one open Proposal at a time for each
        // role.
        Status status = 7;

        // The human-readable terms of transfer.
        string terms = 8;
    }

Proposals with the same address are stored in a list sorted
alphabetically first by ``record_id``, then by ``receiving_agent``,
then by ``timestamp`` (earliest to latest).

.. code-block:: protobuf

    message ProposalList {
        repeated Proposal entries = 1;
    }

.. _Addressing:

Addressing
----------

Grid Track and Trace objects are stored under the namespace obtained by taking
the first six characters of the SHA-512 hash of the string
``grid_track_and_trace``:

.. code-block:: pycon

   >>> def get_hash(string):
   ...     return hashlib.sha512(string.encode('utf-8')).hexdigest()
   ...
   >>> get_hash('grid_track_and_trace')[:6]
   'a43b46'

After its namespace prefix, the next two characters of a Grid Track and Trace
object's address are a string based on the object's type:

- Property / PropertyPage: ``ea``
- Proposal: ``aa``
- Record: ``ec``

The remaining 62 characters of an object's address are determined by
its type:

- Property: the concatenation of the following:

  - The first 36 characters of the hash of the identifier of its
    associated Record plus the first 22 characters of the hash of its
    Property name.
  - The string ``0000``.

- PropertyPage: the address of the page to which updates are to be
  written is the concatenation of the following:

  - The first 36 characters of the hash of the identifier of its
    associated Record.
  - The first 22 characters of the hash of its Property name.
  - The hex representation of the ``current_page`` of its associated
    Property left-padded to length 4 with 0s.

- Proposal: the concatenation of the following:

  - The first 36 characters of the hash of the identifier of
    its associated Record.
  - The first 22 characters of its ``receiving_agent``.
  - The first 4 characters of the hash of its ``timestamp``.

- Record: the first 62 characters of the hash of its identifier.

For example, if ``fish-456`` is a Record with a ``temperature``
Property and a ``current_page`` of 28, the address for that
PropertyPage is:

.. code-block:: pycon

    >>> get_hash('grid_track_and_trace')[:6] + 'ea'  + get_hash('fish-456')[:36] + get_hash('temperature')[:22] + hex(28)[2:].zfill(4)
    'a43b46ea840d00edc7507ed05cfb86938e3624ada6c7f08bfeb8fd09b963f81f9d001c'


Transactions
============

Transaction Payload
-------------------

All Grid Track and Trace transactions are wrapped in a tagged payload object to
allow for the transaction to be dispatched to appropriate handling logic.

.. code-block:: protobuf

    message TrackAndTracePayload {
        enum Action {
            UNSET_ACTION = 0;
            CREATE_RECORD = 1;
            FINALIZE_RECORD = 2;
            UPDATE_PROPERTIES = 3;
            CREATE_PROPOSAL = 4;
            ANSWER_PROPOSAL = 5;
            REVOKE_REPORTER = 6;
        }

        Action action = 1;

        // The approximate time this payload was submitted, as a Unix UTC timestamp.
        uint64 timestamp = 2;

        // The transaction handler will read from just one of these fields
        // according to the Action.
        CreateRecordAction create_record = 3;
        FinalizeRecordAction finalize_record = 4;
        UpdatePropertiesAction update_properties = 6;
        CreateProposalAction create_proposal = 7;
        AnswerProposalAction answer_proposal = 8;
        RevokeReporterAction revoke_reporter = 9;
    }

Any transaction is invalid if its timestamp is greater than the
validator's system time.

.. _CreateRecord:

Create Record
-------------

When an Agent creates a Record, the Record is initialized with that
Agent as both owner and custodian. Any Properties required of the
Record by its Schema must have initial values provided.

.. code-block:: protobuf

    message CreateRecordAction {
        // The natural key of the Record
        string record_id = 1;

        // The name of the Schema this Record belongs to
        string schema = 2;

        repeated PropertyValue properties = 3;
    }


A CreateRecord transaction is invalid if one of the following
conditions occurs:

- The signer is not registered as a Pike Agent.
- The identifier is the empty string.
- The identifier belongs to an existing Record.
- A valid Schema is not specified.
- Initial values are not provided for all of the Properties specified
  as required by the Schema.
- Initial values of the wrong type are provided.


Finalize Record
---------------

A FinalizeRecord Transaction sets a Record’s ``final`` flag to true. A
finalized Record and its Properties cannot be updated. A Record cannot
be finalized except by its owner, and cannot be finalized if the owner
and custodian are not the same.

.. code-block:: protobuf

    message FinalizeRecordAction {
        // The natural key of the Record
        string record_id = 1;
    }


A FinalizeRecord transaction is invalid if one of the following
conditions occurs:

- The Record it targets does not exist.
- The Record it targets is already final.
- The signer is not both the Record's owner and custodian.


Update Properties
-----------------

An UpdateProperties transaction contains a ``record_id`` and a list of
PropertyValues (see CreateRecord_ above). It can only be (validly)
sent by an Agent authorized to report on the Property.

.. code-block:: protobuf

    message UpdatePropertiesAction {
        // The natural key of the Record
        string record_id = 1;

        repeated PropertyValue properties = 2;
    }


An UpdateProperties transaction is invalid if one of the following
conditions occurs:

- The Record does not exist.
- The Record is final.
- Its signer is not authorized to report on any of the provided properties.
- Any of the provided PropertyValues do not match the types specified in the
  Record's Schema.
- Any of the provided PropertyValue's data types do not match the data type
  specified in the PropertyDefinition.


Create Proposal
---------------

A CreateProposal transaction creates an open Proposal concerning some
Record from the signer to the receiving Agent. This Proposal can be
for transfer of ownership, transfer of custodianship, or authorization
to report. If it is a reporter authorization Proposal, a nonempty list
of Property names must be included.

.. code-block:: protobuf

    message CreateProposalAction {
        // The natural key of the Record
        string record_id = 1;

        // the public key of the Agent to whom the Proposal is sent
        // (must be different from the Agent creating the Proposal)
        string receiving_agent = 2;

        Proposal.Role role = 3;

        repeated string properties = 4;

        // The human-readable terms of transfer.
        string terms = 5;
    }


A CreateProposal transaction is invalid if one of the following
conditions occurs:

- The issuing Agent is not registered.
- The receiving Agent is not registered.
- There is already an open Proposal for the Record and receiving Agent
  for the specified role.
- The Record does not exist.
- The Record is final.
- The signer is not the owner and the Proposal is for transfer of
  ownership or reporter authorization.
- The signer is not the custodian and the Proposal is for transfer of
  custodianship.
- The Proposal is for reporter authorization and the list of Property
  names is empty.


Answer Proposal
---------------

An Agent who is the receiving Agent for a Proposal for some Record can
accept or reject that Proposal, marking the Proposal's status as
``accepted`` or ``rejected``. The Proposal's ``issuing_agent`` cannot
accept or reject it, but can cancel it. This will mark the Proposal's
status as ``canceled`` rather than ``rejected``.

.. code-block:: protobuf

    message AnswerProposalAction {
        enum Response {
            ACCEPT = 0;
            REJECT = 1;
            CANCEL = 2;
        }

        // The natural key of the Record
        string record_id = 1;

        // The public key of the Agent to whom the proposal is sent
        string receiving_agent = 2;

        // The role being proposed (owner, custodian, or reporter)
        Proposal.Role role = 3;

        // The respose to the Proposal (accept, reject, or cancel)
        Response response = 4;
    }


Proposals can conflict, in the sense that a Record's owner might have
opened ownership transfer Proposals with several Agents at once. These
Proposals will not be closed if one of them is accepted. Instead, an
``accept`` answer will check to verify that the issuing Agent is still
the owner or custodian of the Record.

An AnswerProposal transaction is invalid if one of the following
conditions occurs:

- There is no Proposal for that receiving agent, record, and role.
- The signer is not the receiving or issuing Agent of the Proposal.
- The signer is the receiving Agent and answers ``cancel``.
- The signer is the issuing Agent and answers anything other than
  ``cancel``.
- The response is ``accept``, but the issuing Agent is no longer the
  owner or custodian (as appropriate to the role) of the Record.
- The referenced record is no longer valid.


Revoke Reporter
---------------

The owner of a Record can send a RevokeReporter transaction to remove
a reporter's authorization to report on one or more Properties for
that Record.

.. code-block:: protobuf

    message RevokeReporterAction {
        // The natural key of the Record
        string record_id = 1;

        // The reporter's public key
        string reporter_id = 2;

        // The names of the Properties for which the reporter's
        // authorization is revoked
        repeated string properties = 3;
    }

A RevokeReporter transaction is invalid if one of the following
conditions occurs:

- The Record does not exist.
- The Record is final.
- The signer is not the Record's owner.
- The reporter whose authorization is to be revoked is not an
  authorized reporter for the Record.
- Any of the provided properties do not exist.

.. Licensed under Creative Commons Attribution 4.0 International License
.. https://creativecommons.org/licenses/by/4.0/
