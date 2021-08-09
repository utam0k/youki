use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::cgroups::common;

use super::controller::Controller;
use oci_spec::{LinuxBlockIo, LinuxResources};

const CGROUP_BFQ_IO_WEIGHT: &str = "io.bfq.weight";
const CGROUP_IO_WEIGHT: &str = "io.weight";

pub struct Io {}

impl Controller for Io {
    fn apply(linux_resource: &LinuxResources, cgroup_root: &Path) -> Result<()> {
        log::debug!("Apply io cgrup v2 config");
        if let Some(io) = &linux_resource.block_io {
            Self::apply(cgroup_root, io)?;
        }
        Ok(())
    }
}

impl Io {
    fn io_max_path(path: &Path) -> PathBuf {
        path.join("io.max")
    }

    // linux kernel doc: https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html#io
    fn apply(root_path: &Path, blkio: &LinuxBlockIo) -> Result<()> {
        if let Some(weight_device) = blkio.weight_device.as_ref() {
            for wd in weight_device {
                common::write_cgroup_file(
                    root_path.join(CGROUP_BFQ_IO_WEIGHT),
                    &format!("{}:{} {}", wd.major, wd.minor, wd.weight.unwrap()),
                )?;
            }
        }
        if let Some(leaf_weight) = blkio.leaf_weight {
            if leaf_weight > 0 {
                bail!("cannot set leaf_weight with cgroupv2");
            }
        }
        if let Some(io_weight) = blkio.weight {
            if io_weight > 0 {
                common::write_cgroup_file(
                    root_path.join(CGROUP_IO_WEIGHT),
                    format!("{}", io_weight),
                )?;
            }
        }

        if let Some(throttle_read_bps_device) = blkio.throttle_read_bps_device.as_ref() {
            for trbd in throttle_read_bps_device {
                common::write_cgroup_file(
                    Self::io_max_path(root_path),
                    &format!("{}:{} rbps={}", trbd.major, trbd.minor, trbd.rate),
                )?;
            }
        }

        if let Some(throttle_write_bps_device) = blkio.throttle_write_bps_device.as_ref() {
            for twbd in throttle_write_bps_device {
                common::write_cgroup_file(
                    Self::io_max_path(root_path),
                    format!("{}:{} wbps={}", twbd.major, twbd.minor, twbd.rate),
                )?;
            }
        }

        if let Some(throttle_read_iops_device) = blkio.throttle_read_iops_device.as_ref() {
            for trid in throttle_read_iops_device {
                common::write_cgroup_file(
                    Self::io_max_path(root_path),
                    format!("{}:{} riops={}", trid.major, trid.minor, trid.rate),
                )?;
            }
        }

        if let Some(throttle_write_iops_device) = blkio.throttle_write_iops_device.as_ref() {
            for twid in throttle_write_iops_device {
                common::write_cgroup_file(
                    Self::io_max_path(root_path),
                    format!("{}:{} wiops={}", twid.major, twid.minor, twid.rate),
                )?;
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::cgroups::test::setup;
    use oci_spec::{LinuxBlockIo, LinuxThrottleDevice, LinuxWeightDevice};
    use std::fs;
    struct BlockIoBuilder {
        block_io: LinuxBlockIo,
    }
    impl BlockIoBuilder {
        fn new() -> Self {
            let block_io = LinuxBlockIo {
                weight: Some(0),
                leaf_weight: Some(0),
                weight_device: vec![].into(),
                throttle_read_bps_device: vec![].into(),
                throttle_write_bps_device: vec![].into(),
                throttle_read_iops_device: vec![].into(),
                throttle_write_iops_device: vec![].into(),
            };

            Self { block_io }
        }
        fn with_write_weight_device(mut self, throttle: Vec<LinuxWeightDevice>) -> Self {
            self.block_io.weight_device = throttle.into();
            self
        }
        fn with_write_io_weight(mut self, iow: u16) -> Self {
            self.block_io.weight = Some(iow);
            self
        }

        fn with_read_bps(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.throttle_read_bps_device = throttle.into();
            self
        }

        fn with_write_bps(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.throttle_write_bps_device = throttle.into();
            self
        }

        fn with_read_iops(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.throttle_read_iops_device = throttle.into();
            self
        }

        fn with_write_iops(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.throttle_write_iops_device = throttle.into();
            self
        }

        fn build(self) -> LinuxBlockIo {
            self.block_io
        }
    }

    #[test]
    fn test_set_io_read_bps() {
        let (tmp, throttle) = setup("test_set_io_read_bps", "io.max");

        let blkio = BlockIoBuilder::new()
            .with_read_bps(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Io::apply(&tmp, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle).unwrap_or_else(|_| panic!("read rbps content"));

        assert_eq!("8:0 rbps=102400", content);
    }

    #[test]
    fn test_set_io_write_bps() {
        let (tmp, throttle) = setup("test_set_io_write_bps", "io.max");

        let blkio = BlockIoBuilder::new()
            .with_write_bps(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Io::apply(&tmp, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle).unwrap_or_else(|_| panic!("read rbps content"));

        assert_eq!("8:0 wbps=102400", content);
    }

    #[test]
    fn test_set_io_read_iops() {
        let (tmp, throttle) = setup("test_set_io_read_iops", "io.max");

        let blkio = BlockIoBuilder::new()
            .with_read_iops(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Io::apply(&tmp, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle).unwrap_or_else(|_| panic!("read riops content"));

        assert_eq!("8:0 riops=102400", content);
    }

    #[test]
    fn test_set_io_write_iops() {
        let (tmp, throttle) = setup("test_set_io_write_iops", "io.max");

        let blkio = BlockIoBuilder::new()
            .with_write_iops(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Io::apply(&tmp, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle).unwrap_or_else(|_| panic!("read wiops content"));

        assert_eq!("8:0 wiops=102400", content);
    }

    #[test]
    fn test_set_ioweight_device() {
        let (tmp, throttle) = setup("test_set_io_weight_device", CGROUP_BFQ_IO_WEIGHT);
        let blkio = BlockIoBuilder::new()
            .with_write_weight_device(vec![LinuxWeightDevice {
                major: 8,
                minor: 0,
                weight: Some(80),
                leaf_weight: Some(0),
            }])
            .build();
        Io::apply(&tmp, &blkio).expect("apply blkio");
        let content =
            fs::read_to_string(throttle).unwrap_or_else(|_| panic!("read bfq_io_weight content"));

        assert_eq!("8:0 80", content);
    }

    #[test]
    fn test_set_ioweight() {
        let (tmp, throttle) = setup("test_set_io_weight", CGROUP_IO_WEIGHT);
        let blkio = BlockIoBuilder::new().with_write_io_weight(100).build();
        Io::apply(&tmp, &blkio).expect("apply blkio");
        let content =
            fs::read_to_string(throttle).unwrap_or_else(|_| panic!("read bfq_io_weight content"));

        assert_eq!("100", content);
    }
}
