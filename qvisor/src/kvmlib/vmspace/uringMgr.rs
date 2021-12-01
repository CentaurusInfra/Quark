// Copyright (c) 2021 Quark Container Authors / 2018 The gVisor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::vec::Vec;

use super::super::qlib::common::*;
use super::super::qlib::uring::sys::sys::*;
use super::super::qlib::uring::*;

use super::super::*;
use super::host_uring::*;

//#[derive(Debug)]
pub struct UringMgr {
    pub uringfds: Vec<i32>,
    pub eventfd: i32,
    pub fds: Vec<i32>,
    pub rings: Vec<IoUring>,
    pub uringSize: usize
}

pub const FDS_SIZE : usize = 8192;

impl UringMgr {
    pub fn New(size: usize) -> Self {
        let eventfd = unsafe {
            libc::eventfd(0, 0)
        };

        if eventfd < 0 {
            panic!("UringMgr eventfd fail {}", errno::errno().0);
        }

        VMSpace::UnblockFd(eventfd);

        let mut fds = Vec::with_capacity(FDS_SIZE);
        for _i in 0..FDS_SIZE {
            fds.push(-1);
        }

        let ret = Self {
            uringfds: Vec::new(),
            eventfd: eventfd,
            fds: fds,
            rings: Vec::new(),
            uringSize: size
        };

        return ret;
    }

    pub fn CompleteLen(&self) -> usize {
        let mut cnt = 0;
        for u in &self.rings {
            cnt += u.CompleteLen();
        }

        return cnt;
    }

    pub fn Eventfd(&self) -> i32 {
        return self.eventfd;
    }

    pub fn Init(&mut self, DedicateUringCnt: usize) {
        if DedicateUringCnt == 0 {
            let ring = Builder::default()
                .setup_sqpoll(10)
                .setup_sqpoll_cpu(0) // vcpu#0
                .setup_clamp()
                .setup_cqsize(self.uringSize as u32 * 2)
                .build(self.uringSize as u32).expect("InitUring fail");
            self.uringfds.push(ring.fd.0);
            self.rings.push(ring);
        } else {
            for i in 0..DedicateUringCnt {
                let ring = Builder::default()
                    .setup_sqpoll(10)
                    .setup_sqpoll_cpu(i as u32)
                    .setup_clamp()
                    .setup_cqsize(self.uringSize as u32 * 2)
                    .build(self.uringSize as u32).expect("InitUring fail");
                self.uringfds.push(ring.fd.0);
                self.rings.push(ring);
            }
        }

        self.Register(IORING_REGISTER_EVENTFD, &self.eventfd as * const _ as u64, 1).expect("InitUring register eventfd fail");
        self.Register(IORING_REGISTER_FILES, &self.fds[0] as * const _ as u64, self.fds.len() as u32).expect("InitUring register files fail");
    }

    pub fn Setup(&mut self, idx: usize, submission: u64, completion: u64) -> Result<i32> {
        self.rings[idx].CopyTo(submission, completion);
        return Ok(0)
    }

    pub fn Enter(&mut self, idx: usize, toSumbit: u32, minComplete:u32, flags: u32) -> Result<i32> {
        let ret = IOUringEnter(self.uringfds[idx], toSumbit, minComplete, flags);
        if ret < 0 {
            return Err(Error::SysError(-ret as i32))
        }

        return Ok(ret as i32)
    }

    pub fn CompletEntries(&self) -> usize {
        let mut cnt = 0;
        for r in &self.rings {
            cnt += r.completion().len();
        };

        return cnt;
    }

    pub fn Wake(&self, idx: usize, minComplete: usize) -> Result<()> {
        let fd = self.uringfds[idx];
        let ret = if minComplete == 0 {
            IOUringEnter(fd, 1, minComplete as u32, IORING_ENTER_SQ_WAKEUP)
        } else {
            IOUringEnter(fd, 1, minComplete as u32, 0)
        };

        //error!("uring wake minComplete {} ret {}, free {}", minComplete, ret, self.ring.sq.freeSlot());
        //self.ring.sq.Print();
        if ret < 0 {
            return Err(Error::SysError(-ret as i32))
        }

        return Ok(());
    }

    pub fn Register(&self, opcode: u32, arg: u64, nrArgs: u32) -> Result<()> {
        for fd in &self.uringfds {
            self.RegisterOne(*fd, opcode, arg, nrArgs)?;
        }

        return Ok(())
    }

    pub fn RegisterOne(&self, fd: i32, opcode: u32, arg: u64, nrArgs: u32) -> Result<()> {
        let ret = IOUringRegister(fd, opcode, arg, nrArgs);
        if ret < 0 {
            error!("IOUringRegister get fail {}", ret);
            return Err(Error::SysError(-ret as i32))
        }

        return Ok(())
    }

    pub fn UnRegisterFile(&mut self) -> Result<()> {
        return self.Register(IORING_UNREGISTER_FILES, 0, 0)
    }

    pub fn Addfd(&mut self, fd: i32) -> Result<()> {
        if fd as usize >= self.fds.len() {
            error!("Addfd out of bound fd {}", fd);
            panic!("Addfd out of bound fd {}", fd)
        }
        self.fds[fd as usize] = fd;

        let fu = sys::io_uring_files_update {
            offset : fd as u32,
            resv: 0,
            fds: self.fds[fd as usize..].as_ptr() as _,
        };

        return self.Register(IORING_REGISTER_FILES_UPDATE, &fu as * const _ as u64, 1);
    }

    pub fn Removefd(&mut self, fd: i32) -> Result<()> {
        if fd as usize >= self.fds.len() {
            error!("Removefd out of bound fd {}", fd);
            panic!("Removefd out of bound fd {}", fd)
        }

        self.fds[fd as usize] = -1;
        let fu = sys::io_uring_files_update {
            offset : fd as u32,
            resv: 0,
            fds: self.fds[fd as usize..].as_ptr() as _,
        };

        return self.Register(IORING_REGISTER_FILES_UPDATE, &fu as * const _ as u64, 1);
    }
}


