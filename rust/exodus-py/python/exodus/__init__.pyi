"""
Type stubs for exodus-py package.
"""

from typing import Any
from . import exomerge as exomerge

# Re-export all from the exodus module
from .exodus import (
    __version__ as __version__,
    EntityType as EntityType,
    CreateMode as CreateMode,
    FloatSize as FloatSize,
    Int64Mode as Int64Mode,
    AttributeType as AttributeType,
    NodeType as NodeType,
    CacheConfig as CacheConfig,
    ChunkConfig as ChunkConfig,
    PerformanceConfig as PerformanceConfig,
    InitParams as InitParams,
    CreateOptions as CreateOptions,
    Block as Block,
    NodeSet as NodeSet,
    SideSet as SideSet,
    EntitySet as EntitySet,
    Assembly as Assembly,
    Blob as Blob,
    QaRecord as QaRecord,
    TruthTable as TruthTable,
    AttributeData as AttributeData,
    ExodusReader as ExodusReader,
    ExodusWriter as ExodusWriter,
    ExodusAppender as ExodusAppender,
    BlockBuilder as BlockBuilder,
    MeshBuilder as MeshBuilder,
)

_exodus_available: bool
