
Flop
====

Flexible Object Programming

Flop is an interpreted, un-traditional object oriented language, with a
built-in event queue for coordinating in-game time.


Features
--------

Statically Typed (In theory...)
No Classes
Interfaces/Traits (called 'roles')
Multiple Implementations of the same role (called "interfaces")
Polymorphism (of course)
Reference counting
Simple type system (numbers, references, sets of references)
Coordinate in-game time with built-in event queue and in-line wait command
Easy algebraic data types without any declarations


Usage
=====

The first thing you need is a program that uses Flop, none of which publically
exist!

Then you make a new type, which is defined in its own file, in the flop
directory of the project.

The type needs at least 3 things to be usable, a role that defines how it can
be used, an implementation that defines the function(s) used by the role, and
an interface which binds those functions to the role for usage.

Syntax Overview
---------------

The first thing to get used to about Flop's syntax is the binding symbols,
loosely inspired by Jai.

`:=` is meant to be used for static 'compile time' bindings, `:` for type
declarations, and `=` for dynamic 'run time' bindings.

There is one exception to this, which may or may not be changed.

Then, also inspired by Jai, is the pattern of these top level bindings:
`name := Kind { body... }`
e.g.
`MyRole := role { fun: action() }`
`my_action := action() { wait(10); }`


Algorithms
----------

Most of the content of Flop programming is writing algorithms/functions.
(these terms are interchangeable)

Currently the two kinds of algorithm are actions and initializers, though once
more static analysis exists, an additional distinction will exist to describe
the extent to which the algorithm can modify its environment.

The two differences between actions and initializers is that actions can pass
control over to another object, and assume that the object is already
initialized, whereas intializers must return control back to their caller, and
assume that the object is __not__ already initialized.

During the course of an algorithm objects can be initialized and uninitialized
many times, but must always be in an initialized state at the end of a program,
and during any `wait` commands.

The definition of an algorithm takes the form:
```
<identifier> := [action | intializer](<identifier> [...]) {
    [<statement>;]
    [...]
}
```

where statement is any of the following:

###Wait

###Initialize State

###Deinitialize State

###Variable assignment

###Set insertion

###Set deletion

###Set iteration

###Action execution

Expressions
-----------

###Numeric literal

###Numeric operation

###Object initializer

###Global function

writing game.function(args...) allows you to call project-specific functions
written in rust.

###Empty set

###Variable

###Copy of Variable

this really should be automatically done
