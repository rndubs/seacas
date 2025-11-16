# Source this file to set up environment for C compatibility tests
#
# Usage:
#   source ./env-compat.sh
#

export EXODUS_DIR="/home/user/seacas/rust/compat-tests/exodus-install"
export TPL_DIR="/home/user/seacas/rust/compat-tests/tpl-install"
export LD_LIBRARY_PATH="/home/user/seacas/rust/compat-tests/exodus-install/lib:/home/user/seacas/rust/compat-tests/tpl-install/lib:$LD_LIBRARY_PATH"
export PATH="/home/user/seacas/rust/compat-tests/exodus-install/bin:$PATH"
export PKG_CONFIG_PATH="/home/user/seacas/rust/compat-tests/tpl-install/lib/pkgconfig:$PKG_CONFIG_PATH"
export HDF5_DIR="/home/user/seacas/rust/compat-tests/tpl-install"
export NETCDF_DIR="/home/user/seacas/rust/compat-tests/tpl-install"

echo "C compatibility test environment configured"
echo "  Exodus library: $EXODUS_DIR"
echo "  TPL libraries:  $TPL_DIR"
