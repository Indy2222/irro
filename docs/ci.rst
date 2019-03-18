======================
Continuous Integration
======================

Continuous integration is based on `Travis CI`_, see ``.travis.yml`` file
placed in the root of the repository. All tests are executed from ``ci/ci.sh``
Bash script which is executed inside Docker image ``miindy/irro-ci:latest``.
The Docker image must be re-build manually with ``ci/push.sh`` script.

.. _Travis CI: https://travis-ci.org/
