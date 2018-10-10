Smart Permissions
*****************

Abstract
========

Supply chain applications require organization-specific permissioning business
logic; however, existing distributed ledger systems do not generally include
a mechanism for representing organization-specific business logic.  This paper
describes a software architecture and design for implementing such
a permissioning system for distributed ledger applications using Hyperledger
Sawtooth and WebAssembly.

Background
==========

Blockchain and distributed ledgers are a natural building block for supply
chain applications, primarily due to cross-organization data sharing with no
central trusted entity.

.. todo::

    High-level description of supply chain applications on distributed ledgers.

.. todo::

    Description of Permissioning within supply chain applications.

.. todo::

    Describe smart contracts and transaction processors.

Smart Permission Functions
==========================

Smart permission functions (SPFs) are business logic implemented in
a programming language, stored in a distributed ledger's global state, and
executed within a smart contract during transaction execution.

In some respects, Smart Permission Functions are similar to smart contracts, in
that both can be stored on the chain, retrieved from global state as needed,
and executed during transaction execution. However, the purposes are distinct.

Smart contracts implement business logic to update global state, and can
generally be considered state transition functions of the form S1 = T(S0).
Smart Permission Functions implement business logic to return a boolean result
which answers a specific permission question: A = F(T, S0, P). This allows
shared/agreed-upon contract logic which governs the mechanics of a transaction
to be gated by a domain or organization-specific set of permissions. A typical
usage of this is an organizational Smart Permission Function called from within
a smart contract.

Implementation and Compilation
------------------------------

The Pike SDK provides an easy framework for implementing SPFs in Rust,
including the SmartPermissionFunction trait.  Other languages may be used for
implementing SPFs as well, as long as they can be compiled into WASM.

.. todo::

    Provide an example SPF implementation here

The above can be compiled using the following commands:

.. todo::

    Instructions for compiling the SPF.

The resulting WASM file contains compiled SPF, which is ready to be submitted
to the Sawtooth network as a transaction.

Storage and Retrieval
---------------------

Smart Permission Functions are stored in Global State.  The pike
command is used to create SPF-related transactions, submit those transactions
to the network, and view the current list of SPFs contained in global state.
Additional user interfaces, such as an identity management web application, may
be added in the future.

A Pike SDK provides a framework for retrieving and evaluating SPFs
within the context of a Sawtooth transaction processor.

Evaluation
----------

An application uses the permission system to answer a boolean question, "Is
the current operation permitted?", with various inputs:

- transactor public key
- permission-specific function parameters
