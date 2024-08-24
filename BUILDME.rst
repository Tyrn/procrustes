Procrustes a.k.a. Damastes
**************************

Audio album builder; copy and edit on the fly.

Development
===========

- `Format Rust code <https://github.com/rust-lang/rustfmt>`__
- `TagLib <https://github.com/taglib/taglib>`__ library, release build, required on the system; `Rust bindings <https://github.com/ebassi/taglib-rust>`__ .
- For the Archlinux family, just check `taglib` which sould be already there:

::

    $ yay -S taglib

- Ubuntu/Debian:

::

    $ sudo apt-get install libtagc0-dev

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
