
Proto-Flop
==========

An embedded language designed for a project that I shouldn't have attempted.

Back when I was designing and attempting to implement a monstrous idea called
Civil, I imagined in-game content being programmed using some modular language
capable of two things:
1. Scheduling objects' actions using a binary heap/BTree of events
2. reusing old code in a static, yet flexible way

Most of the time I would focus on one of these goals or the other, to the point
where I barely realized I had two distinct goals.

(This would be a recurring theme in Civil's design, but the embedded language
is the only thing I've ever tried implementing, this is the second attempt in
fact)

At this point proto-flop is mangled and would require an overhaul to remove
traces of either of the above goals.

To some extent I designed goal 2 and implemented goal 1. What follows is a
very high level overview of the design concepts associated with each goal.


State Based Temporal Procedures
===============================

The first goal of Civil's embedded language was to be convenient for event
queues, which would in turn be convenient for low density high volume
simulations. (Simulating 1000 entities that only update once per minute each,
or 100 entities at highly accelerated simulation speed)

The two concepts that seemed to make working with queued events easiest were:
1. State Machines/Algebraic Data Types
2. Procedural programming with inline 'wait' commands

these formed the core of proto-flop, and would make a strong core for something
similar, if one should ever need an embedded language for configuring a
low-density game simulation.

One idea that seemed particularly fruitful was to associate each 'wait'
invocation with a dedicated state, that unlike others, could only be bound
immediately before that wait command.
This meant that syntax for cancelling the current procedure was simply that of
overriding the current state, and any variables in the context of the wait
command could be extracted elsewhere for recovering from interruption.

Proto-flop was loosely object oriented as a result of its other design goal,
and so it was simplest to associate each state to a particular class, and each
object would inhabit a single state of those associated with its class,
corresponding to zero or one wait events depending on the state.
This made proto-flop very elegant, though its usefulness in other
implementations may vary.


Role Based Polymorphism
=======================

The name Flop was conceived of once I incorporated these concepts.
Flop stands for FLexible Object Programming, since the core idea here is that
the same abstract class can be 'inherited from' or implemented multiple times
at once, which means old implementations do not interfere with new.

To me role-interface systems are crucial for any polymorphic or generic
programming to be robust, which is unfortunate for all existing static
languages... except ML

The goal of Flop now is to be a robust embedded language, to take an object
oriented paradigm more powerful than C++, and to use it to constrain embedded
code more static/verifiable than Lua or Python.
That said I still do not have a concrete use case at the time of writing, so
this design is also up for debate.

Flop has a lot of concepts, most of which are at least analogous to a
mainstream OOP idea:

Class
-----

A class is a user-defined compound data type with attached methods for
convenient usage, as always.

Unlike other languages all methods and fields are private unless opted into
downstream, (I imagine via semantic versioning) or put into an interface...

`class X { fun(){}... }`

Interface
---------

This is _not_ like an abstract class. An interface is a bundle of public
methods, similar to how the term is used in C or perhaps even in physical
hardware.

Interfaces are explicit blocks of code inside of a class, and methods in an
interface can and will conflict with methods in other interfaces or within the
class itself.

`class X { interface X { fun(){}... } }`

In combination with roles Flop programs will naturally adhere to dependency
inversion, thus the focus on interfaces for public methods, despite their later
focus on implementation of a role.

Role
----

A role is a named signature describing how an interface could potentially look.

It is like an abstract class, an interface, a trait, a typeclass, or a
signature, in other languages.

Because interfaces are treated as (compile-time) objects in their own right,
roles are simply a description of an interface, and not an "abstract
superclass" of an interface like in traditional OOP.

`role R { fun() }`
`class X { R X { fun(){} } }`

All interfaces invoked with the `interface` command will have an implicit role
with the same name, and hence roles and interfaces must exist in their own
namespaces to deal with this.

Data Types
----------

Classes and Roles (and hence anonymous Interfaces) each correspond to a data
type, namely pointer to class, and pointer to polymorphic class respectively.

Presumably most use cases will require some form of number type, algebraic data
type, and array type.

As most modern languages do, one should avoid nullable pointers, and instead
use optional types which optimize when applied to pointers.

Methods and Constructors
------------------------

Methods are pretty standard, they take inputs and produce outputs, while
reading and/or mutating an object.

Ideally multiple return values would be possible without the use of tuple
types, in the same way that multiple input values are possible.

Constructors are functions that when implemented take an uninitialized object
and initialize it, and when called return a newly initialized object.

They can be freely attached to interfaces/roles as a sensible alternative to
factory methods.

Static/Constructor Role
-----------------------

Each role should have an associated 'constructor role' which treats all
constructors (and static functions) of the old role, as methods of objects that
implement this new role.

Each interface should be implicitly usable as a constructor object, though the
constructor role could be directly implemented as well.

This serves as an alternative to abstract factory classes, though those aren't
that bad... maybe this feature isn't necessary.

constructor objects could be implemented as polymorphic pointers with a nulled
data and a non-nulled vtable.


Function Role
-------------

Every function should correspond to 


Explicit Subroles
-----------------

There are many interesting subsets possible in this framework and similar
frameworks.

The most normal of these is an explicit subrole:
```
role Super {}
role Sub : Super {}

class X {
    XSuper {}
    XSub : XSuper {}
    XSub2 {}  // Super has no methods but if it did they'd be here as well
}
```

Ignoring complications such as Design-by-Contract, this pattern should
naturally enforce Liskov substitution, i.e....

Subtypes
--------

Pointers to subroles are subtypes if the roles are subroles, thus Liskov
Substitution.

ADTs with missing variants or extra fields are potentially subtypes, though
this might require an implict conversion/coersion at runtime.

Beyond that variance rules should apply, so arrays, ADTs, and function results
should respect subtype rules, whereas function inputs should invert subtype
rules.

Implicit Subroles
-----------------

This may not be the best idea, but adding methods or generalizing a method
could create an implicit subrole, though adding methods probably ought to be
explicit so that vtable ordering is consistent.

```
role Super { fun produce() -> Apple }
role Sub { fun produce() -> Fruit }  // no `:` but still a subtype
```

in this way explicit subroles and subinterfaces could be treated as syntax
sugar for this style of rubrole.

Explicit Superroles
-------------------

In the spirit of interface segregation, (well more in the spirit of ignoring
it) it could be useful to exclude certain methods from a role, while still
accepting full implementors of that role, particularly if you do not have
access to the role.

```
role Role { fun a(), fun b() }
fun invoke_a(x: Role - b) { x.a(); }
```

Downstream Interfaces
---------------------

Given the flexibility of named interfaces, it is entirely possible for one
class or package to define interfaces on another class, though it will
typically need to do this using public interfaces.

Connected to this is potential functors between roles.

Optional/Downstream Methods
---------------------------

In the spirit of the Open/Closed principle, roles could have optional methods
added to them, with a default implementation for when they aren't implemented.

In the spirit of downstream interfaces, existing roles could be leveraged for
communication between downstream objects by adding optional methods downstream.

These downstream methods will somehow be invoked based on their full scope,
similar to Rust's `use module::module::Trait;` semantics.


This idea came directly from imagined scenarios in modifying Civil based games.

One could take an existing game with its own role for communication between
objects, then you could add your own method for a signal that your objects can
respond to but most other objects ignore... e.g. a new tool that can interact
with new items.

(incomplete ADTs with downstream variants might also be useful in the same
situations)

Notes about pointers
--------------------

In order for default/downstream interfaces and methods to be usable
interchangeably with interfaces defined with private access to a class, all
methods will need to be passed a full polymorphic pointer with irrelevant parts
nulled.

Similarly constructors will need to be passed an interface/constructor object
so that they can wrap the appropriate interface around the result... or just do
that at the call-site of the constructor

this does impose limitations on how subtypes can work, for example you can't
make a generic `A + B` role that can be coerced into `A` _and_ `B` because of
vtable alignment... assuming that `A + B` is defined by writing a functor from
`A` to `B` anyway

Notes about genericity
----------------------

Making classes and roles that depend on types, classes, interfaces, or values,
could allow for powerful genericity and/or manual optimization of virtual
function calls, though this begins to look less like an OOP language, in which
modularity should be top priority, even if it means extra pointer chasing.
(please no SFINAE though.....)

Additionally agda style instance variables that provide a default coercion from
class to role could be useful, it does make existing languages more succinct
than flop code, but it would be worth trying the language without this feature
for a while just to see if it is important... it is more useful for genericity
than polymorphism so it could depend a lot on the above.

