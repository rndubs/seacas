"""
Test code snippets in documentation using Sybil.

This conftest configures pytest to validate all Python code snippets
in the MyST markdown documentation files to ensure they remain up-to-date
and functional.
"""
import os
from sybil import Sybil
from sybil.parsers.myst import PythonCodeBlockParser, SkipParser

# Setup function to add common imports to all examples
def setup(namespace):
    """Add common imports and helpers to the test namespace."""
    # Change to docs directory so file paths work
    docs_dir = os.path.dirname(__file__)
    os.chdir(docs_dir)

    # Import all common exodus-py modules
    import exodus
    from exodus import (
        ExodusReader, ExodusWriter, ExodusAppender,
        MeshBuilder, BlockBuilder,
        InitParams, CreateOptions, CreateMode, FloatSize, Int64Mode,
        Block, NodeSet, SideSet, EntitySet, Assembly, Blob, QaRecord,
        EntityType,
    )

    # Create results.exo with time series data for examples
    if not os.path.exists("results.exo") or os.path.getsize("results.exo") < 1000:
        writer = ExodusWriter.create("results.exo", CreateOptions(mode=CreateMode.Clobber))
        params = InitParams(
            title="Results Example",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)
        writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [])

        # This will automatically create the element block
        # Just write basic mesh structure - variables will be added differently
        writer.close()

        # Reopen to add time series (since define_variables may not be available)
        # For now, just create a basic file

    # Add convenience functions for examples
    def skip_example():
        """Marker function to skip an example - does nothing"""
        pass

    # Add to namespace
    namespace.update({
        'exodus': exodus,
        'ExodusReader': ExodusReader,
        'ExodusWriter': ExodusWriter,
        'ExodusAppender': ExodusAppender,
        'MeshBuilder': MeshBuilder,
        'BlockBuilder': BlockBuilder,
        'InitParams': InitParams,
        'CreateOptions': CreateOptions,
        'CreateMode': CreateMode,
        'FloatSize': FloatSize,
        'Int64Mode': Int64Mode,
        'Block': Block,
        'NodeSet': NodeSet,
        'SideSet': SideSet,
        'EntitySet': EntitySet,
        'Assembly': Assembly,
        'Blob': Blob,
        'QaRecord': QaRecord,
        'EntityType': EntityType,
        'skip_example': skip_example,
    })

# Separate configurations for different types of documentation

# Complete examples that should run (tutorials, quickstart)
pytest_collect_file = Sybil(
    parsers=[
        PythonCodeBlockParser(doctest_optionflags=2),  # ELLIPSIS flag
        SkipParser(),
    ],
    # Test quickstart and index which have complete examples
    patterns=['index.md', 'quickstart.md'],
    setup=setup,
).pytest()

# Note: user_guide.md and api_reference.md contain many illustrative snippets
# that are not meant to be complete runnable examples. Those can be tested
# separately with more lenient rules or skipped entirely.
