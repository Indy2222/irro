======================
Continuous Integration
======================

Continuous integration is based on `Travis CI`_, see ``.travis.yml`` file
placed in the root of the repository. All tests are executed from ``ci/ci.sh``
Bash script which is executed inside Docker image ``miindy/irro-ci:latest``.
The Docker image must be re-build manually with ``ci/push.sh`` script.

.. _Travis CI: https://travis-ci.org/

Travis automatically deploys built documentation to irro.mgn.cz (this site)
whenever ``master`` branch changes. For it to work correctly, ``GITHUB_TOKEN``
environment variable is configured_ for the Travis CI repository.

.. _configured: https://docs.travis-ci.com/user/environment-variables/#defining-variables-in-repository-settings

Travis also automatically uploads build artifacts to `Google Cloud Storage
(GCS)`_ bucket `ci.gs.irro.mgn.cz`_. Travis authenticates to GCS with
``GCS_ID`` and ``GCS_KEY`` environment variables. The access key can be
obtained in `GCS interoperability`_ settings.

.. _Google Cloud Storage (GCS): https://cloud.google.com/storage/

.. _ci.gs.irro.mgn.cz: https://storage.googleapis.com/ci.gs.irro.mgn.cz/

.. _GCS interoperability: https://cloud.google.com/storage/docs/interoperability
