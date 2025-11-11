# Source this file to set up environment for C compatibility tests
#
# Usage:
#   source ./env-compat.sh
#

export EXODUS_DIR="/home/user/seacas/build-compat/install"
export LD_LIBRARY_PATH="$EXODUS_DIR/lib:/home/user/seacas/TPL/netcdf-4.9.2/lib:/home/user/seacas/TPL/hdf5-1.14.6/lib:$LD_LIBRARY_PATH"
export PATH="$EXODUS_DIR/bin:$PATH"

echo "C compatibility test environment configured"
echo "  Exodus library: $EXODUS_DIR"
