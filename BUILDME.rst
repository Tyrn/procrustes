Procrustes a.k.a. Damastes
**************************

Audio album builder; copy and edit on the fly.

Development
===========

- `Format Rust code <https://github.com/rust-lang/rustfmt>`__
- `TagLib <https://github.com/taglib/taglib>`__ library, release build, required on the system; `Rust bindings <https://github.com/ebassi/taglib-rust>`__ .
- Arch Linux family:

::

    $ yay -S taglib1

- Ubuntu/Debian:

::

    $ sudo apt-get install libtagc0-dev

- Fedora:

::

    $ sudo dnf install taglib-devel

Build
-----

::

    $ cargo build [--release]

Format
------

::

    $ cargo fmt

Test
----

::

    $ cargo test

Install
-------

Install to ``~/.cargo/bin``:

::

    $ cargo install --locked --path . [--debug]

Publish
-------

::

    TODO
