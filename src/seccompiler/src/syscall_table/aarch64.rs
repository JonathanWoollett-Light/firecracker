// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

// This file is auto-generated by `tools/devtool generate_syscall_tables`.
// Do NOT manually edit!
// Generated at: Mon 15 Nov 11:41:50 UTC 2021
// Kernel version: 5.10

use std::collections::HashMap;

#[tracing::instrument(level = "trace", skip(map))]
pub(crate) fn make_syscall_table(map: &mut HashMap<String, i64>) {
    map.insert("accept4".to_string(), 242);
    map.insert("accept".to_string(), 202);
    map.insert("acct".to_string(), 89);
    map.insert("add_key".to_string(), 217);
    map.insert("adjtimex".to_string(), 171);
    map.insert("bind".to_string(), 200);
    map.insert("bpf".to_string(), 280);
    map.insert("brk".to_string(), 214);
    map.insert("capget".to_string(), 90);
    map.insert("capset".to_string(), 91);
    map.insert("chdir".to_string(), 49);
    map.insert("chroot".to_string(), 51);
    map.insert("clock_adjtime".to_string(), 266);
    map.insert("clock_getres".to_string(), 114);
    map.insert("clock_gettime".to_string(), 113);
    map.insert("clock_nanosleep".to_string(), 115);
    map.insert("clock_settime".to_string(), 112);
    map.insert("clone3".to_string(), 435);
    map.insert("clone".to_string(), 220);
    map.insert("close_range".to_string(), 436);
    map.insert("close".to_string(), 57);
    map.insert("connect".to_string(), 203);
    map.insert("copy_file_range".to_string(), 285);
    map.insert("delete_module".to_string(), 106);
    map.insert("dup3".to_string(), 24);
    map.insert("dup".to_string(), 23);
    map.insert("epoll_create1".to_string(), 20);
    map.insert("epoll_ctl".to_string(), 21);
    map.insert("epoll_pwait".to_string(), 22);
    map.insert("eventfd2".to_string(), 19);
    map.insert("execveat".to_string(), 281);
    map.insert("execve".to_string(), 221);
    map.insert("exit_group".to_string(), 94);
    map.insert("exit".to_string(), 93);
    map.insert("faccessat2".to_string(), 439);
    map.insert("faccessat".to_string(), 48);
    map.insert("fadvise64".to_string(), 223);
    map.insert("fallocate".to_string(), 47);
    map.insert("fanotify_init".to_string(), 262);
    map.insert("fanotify_mark".to_string(), 263);
    map.insert("fchdir".to_string(), 50);
    map.insert("fchmodat".to_string(), 53);
    map.insert("fchmod".to_string(), 52);
    map.insert("fchownat".to_string(), 54);
    map.insert("fchown".to_string(), 55);
    map.insert("fcntl".to_string(), 25);
    map.insert("fdatasync".to_string(), 83);
    map.insert("fgetxattr".to_string(), 10);
    map.insert("finit_module".to_string(), 273);
    map.insert("flistxattr".to_string(), 13);
    map.insert("flock".to_string(), 32);
    map.insert("fremovexattr".to_string(), 16);
    map.insert("fsconfig".to_string(), 431);
    map.insert("fsetxattr".to_string(), 7);
    map.insert("fsmount".to_string(), 432);
    map.insert("fsopen".to_string(), 430);
    map.insert("fspick".to_string(), 433);
    map.insert("fstatfs".to_string(), 44);
    map.insert("fstat".to_string(), 80);
    map.insert("fsync".to_string(), 82);
    map.insert("ftruncate".to_string(), 46);
    map.insert("futex".to_string(), 98);
    map.insert("getcpu".to_string(), 168);
    map.insert("getcwd".to_string(), 17);
    map.insert("getdents64".to_string(), 61);
    map.insert("getegid".to_string(), 177);
    map.insert("geteuid".to_string(), 175);
    map.insert("getgid".to_string(), 176);
    map.insert("getgroups".to_string(), 158);
    map.insert("getitimer".to_string(), 102);
    map.insert("get_mempolicy".to_string(), 236);
    map.insert("getpeername".to_string(), 205);
    map.insert("getpgid".to_string(), 155);
    map.insert("getpid".to_string(), 172);
    map.insert("getppid".to_string(), 173);
    map.insert("getpriority".to_string(), 141);
    map.insert("getrandom".to_string(), 278);
    map.insert("getresgid".to_string(), 150);
    map.insert("getresuid".to_string(), 148);
    map.insert("getrlimit".to_string(), 163);
    map.insert("get_robust_list".to_string(), 100);
    map.insert("getrusage".to_string(), 165);
    map.insert("getsid".to_string(), 156);
    map.insert("getsockname".to_string(), 204);
    map.insert("getsockopt".to_string(), 209);
    map.insert("gettid".to_string(), 178);
    map.insert("gettimeofday".to_string(), 169);
    map.insert("getuid".to_string(), 174);
    map.insert("getxattr".to_string(), 8);
    map.insert("init_module".to_string(), 105);
    map.insert("inotify_add_watch".to_string(), 27);
    map.insert("inotify_init1".to_string(), 26);
    map.insert("inotify_rm_watch".to_string(), 28);
    map.insert("io_cancel".to_string(), 3);
    map.insert("ioctl".to_string(), 29);
    map.insert("io_destroy".to_string(), 1);
    map.insert("io_getevents".to_string(), 4);
    map.insert("io_pgetevents".to_string(), 292);
    map.insert("ioprio_get".to_string(), 31);
    map.insert("ioprio_set".to_string(), 30);
    map.insert("io_setup".to_string(), 0);
    map.insert("io_submit".to_string(), 2);
    map.insert("io_uring_enter".to_string(), 426);
    map.insert("io_uring_register".to_string(), 427);
    map.insert("io_uring_setup".to_string(), 425);
    map.insert("kcmp".to_string(), 272);
    map.insert("kexec_file_load".to_string(), 294);
    map.insert("kexec_load".to_string(), 104);
    map.insert("keyctl".to_string(), 219);
    map.insert("kill".to_string(), 129);
    map.insert("lgetxattr".to_string(), 9);
    map.insert("linkat".to_string(), 37);
    map.insert("listen".to_string(), 201);
    map.insert("listxattr".to_string(), 11);
    map.insert("llistxattr".to_string(), 12);
    map.insert("lookup_dcookie".to_string(), 18);
    map.insert("lremovexattr".to_string(), 15);
    map.insert("lseek".to_string(), 62);
    map.insert("lsetxattr".to_string(), 6);
    map.insert("madvise".to_string(), 233);
    map.insert("mbind".to_string(), 235);
    map.insert("membarrier".to_string(), 283);
    map.insert("memfd_create".to_string(), 279);
    map.insert("migrate_pages".to_string(), 238);
    map.insert("mincore".to_string(), 232);
    map.insert("mkdirat".to_string(), 34);
    map.insert("mknodat".to_string(), 33);
    map.insert("mlock2".to_string(), 284);
    map.insert("mlockall".to_string(), 230);
    map.insert("mlock".to_string(), 228);
    map.insert("mmap".to_string(), 222);
    map.insert("mount".to_string(), 40);
    map.insert("move_mount".to_string(), 429);
    map.insert("move_pages".to_string(), 239);
    map.insert("mprotect".to_string(), 226);
    map.insert("mq_getsetattr".to_string(), 185);
    map.insert("mq_notify".to_string(), 184);
    map.insert("mq_open".to_string(), 180);
    map.insert("mq_timedreceive".to_string(), 183);
    map.insert("mq_timedsend".to_string(), 182);
    map.insert("mq_unlink".to_string(), 181);
    map.insert("mremap".to_string(), 216);
    map.insert("msgctl".to_string(), 187);
    map.insert("msgget".to_string(), 186);
    map.insert("msgrcv".to_string(), 188);
    map.insert("msgsnd".to_string(), 189);
    map.insert("msync".to_string(), 227);
    map.insert("munlockall".to_string(), 231);
    map.insert("munlock".to_string(), 229);
    map.insert("munmap".to_string(), 215);
    map.insert("name_to_handle_at".to_string(), 264);
    map.insert("nanosleep".to_string(), 101);
    map.insert("newfstatat".to_string(), 79);
    map.insert("nfsservctl".to_string(), 42);
    map.insert("openat2".to_string(), 437);
    map.insert("openat".to_string(), 56);
    map.insert("open_by_handle_at".to_string(), 265);
    map.insert("open_tree".to_string(), 428);
    map.insert("perf_event_open".to_string(), 241);
    map.insert("personality".to_string(), 92);
    map.insert("pidfd_getfd".to_string(), 438);
    map.insert("pidfd_open".to_string(), 434);
    map.insert("pidfd_send_signal".to_string(), 424);
    map.insert("pipe2".to_string(), 59);
    map.insert("pivot_root".to_string(), 41);
    map.insert("pkey_alloc".to_string(), 289);
    map.insert("pkey_free".to_string(), 290);
    map.insert("pkey_mprotect".to_string(), 288);
    map.insert("ppoll".to_string(), 73);
    map.insert("prctl".to_string(), 167);
    map.insert("pread64".to_string(), 67);
    map.insert("preadv2".to_string(), 286);
    map.insert("preadv".to_string(), 69);
    map.insert("prlimit64".to_string(), 261);
    map.insert("process_madvise".to_string(), 440);
    map.insert("process_vm_readv".to_string(), 270);
    map.insert("process_vm_writev".to_string(), 271);
    map.insert("pselect6".to_string(), 72);
    map.insert("ptrace".to_string(), 117);
    map.insert("pwrite64".to_string(), 68);
    map.insert("pwritev2".to_string(), 287);
    map.insert("pwritev".to_string(), 70);
    map.insert("quotactl".to_string(), 60);
    map.insert("readahead".to_string(), 213);
    map.insert("readlinkat".to_string(), 78);
    map.insert("read".to_string(), 63);
    map.insert("readv".to_string(), 65);
    map.insert("reboot".to_string(), 142);
    map.insert("recvfrom".to_string(), 207);
    map.insert("recvmmsg".to_string(), 243);
    map.insert("recvmsg".to_string(), 212);
    map.insert("remap_file_pages".to_string(), 234);
    map.insert("removexattr".to_string(), 14);
    map.insert("renameat2".to_string(), 276);
    map.insert("renameat".to_string(), 38);
    map.insert("request_key".to_string(), 218);
    map.insert("restart_syscall".to_string(), 128);
    map.insert("rseq".to_string(), 293);
    map.insert("rt_sigaction".to_string(), 134);
    map.insert("rt_sigpending".to_string(), 136);
    map.insert("rt_sigprocmask".to_string(), 135);
    map.insert("rt_sigqueueinfo".to_string(), 138);
    map.insert("rt_sigreturn".to_string(), 139);
    map.insert("rt_sigsuspend".to_string(), 133);
    map.insert("rt_sigtimedwait".to_string(), 137);
    map.insert("rt_tgsigqueueinfo".to_string(), 240);
    map.insert("sched_getaffinity".to_string(), 123);
    map.insert("sched_getattr".to_string(), 275);
    map.insert("sched_getparam".to_string(), 121);
    map.insert("sched_get_priority_max".to_string(), 125);
    map.insert("sched_get_priority_min".to_string(), 126);
    map.insert("sched_getscheduler".to_string(), 120);
    map.insert("sched_rr_get_interval".to_string(), 127);
    map.insert("sched_setaffinity".to_string(), 122);
    map.insert("sched_setattr".to_string(), 274);
    map.insert("sched_setparam".to_string(), 118);
    map.insert("sched_setscheduler".to_string(), 119);
    map.insert("sched_yield".to_string(), 124);
    map.insert("seccomp".to_string(), 277);
    map.insert("semctl".to_string(), 191);
    map.insert("semget".to_string(), 190);
    map.insert("semop".to_string(), 193);
    map.insert("semtimedop".to_string(), 192);
    map.insert("sendfile".to_string(), 71);
    map.insert("sendmmsg".to_string(), 269);
    map.insert("sendmsg".to_string(), 211);
    map.insert("sendto".to_string(), 206);
    map.insert("setdomainname".to_string(), 162);
    map.insert("setfsgid".to_string(), 152);
    map.insert("setfsuid".to_string(), 151);
    map.insert("setgid".to_string(), 144);
    map.insert("setgroups".to_string(), 159);
    map.insert("sethostname".to_string(), 161);
    map.insert("setitimer".to_string(), 103);
    map.insert("set_mempolicy".to_string(), 237);
    map.insert("setns".to_string(), 268);
    map.insert("setpgid".to_string(), 154);
    map.insert("setpriority".to_string(), 140);
    map.insert("setregid".to_string(), 143);
    map.insert("setresgid".to_string(), 149);
    map.insert("setresuid".to_string(), 147);
    map.insert("setreuid".to_string(), 145);
    map.insert("setrlimit".to_string(), 164);
    map.insert("set_robust_list".to_string(), 99);
    map.insert("setsid".to_string(), 157);
    map.insert("setsockopt".to_string(), 208);
    map.insert("set_tid_address".to_string(), 96);
    map.insert("settimeofday".to_string(), 170);
    map.insert("setuid".to_string(), 146);
    map.insert("setxattr".to_string(), 5);
    map.insert("shmat".to_string(), 196);
    map.insert("shmctl".to_string(), 195);
    map.insert("shmdt".to_string(), 197);
    map.insert("shmget".to_string(), 194);
    map.insert("shutdown".to_string(), 210);
    map.insert("sigaltstack".to_string(), 132);
    map.insert("signalfd4".to_string(), 74);
    map.insert("socketpair".to_string(), 199);
    map.insert("socket".to_string(), 198);
    map.insert("splice".to_string(), 76);
    map.insert("statfs".to_string(), 43);
    map.insert("statx".to_string(), 291);
    map.insert("swapoff".to_string(), 225);
    map.insert("swapon".to_string(), 224);
    map.insert("symlinkat".to_string(), 36);
    map.insert("sync_file_range".to_string(), 84);
    map.insert("syncfs".to_string(), 267);
    map.insert("sync".to_string(), 81);
    map.insert("sysinfo".to_string(), 179);
    map.insert("syslog".to_string(), 116);
    map.insert("tee".to_string(), 77);
    map.insert("tgkill".to_string(), 131);
    map.insert("timer_create".to_string(), 107);
    map.insert("timer_delete".to_string(), 111);
    map.insert("timerfd_create".to_string(), 85);
    map.insert("timerfd_gettime".to_string(), 87);
    map.insert("timerfd_settime".to_string(), 86);
    map.insert("timer_getoverrun".to_string(), 109);
    map.insert("timer_gettime".to_string(), 108);
    map.insert("timer_settime".to_string(), 110);
    map.insert("times".to_string(), 153);
    map.insert("tkill".to_string(), 130);
    map.insert("truncate".to_string(), 45);
    map.insert("umask".to_string(), 166);
    map.insert("umount2".to_string(), 39);
    map.insert("uname".to_string(), 160);
    map.insert("unlinkat".to_string(), 35);
    map.insert("unshare".to_string(), 97);
    map.insert("userfaultfd".to_string(), 282);
    map.insert("utimensat".to_string(), 88);
    map.insert("vhangup".to_string(), 58);
    map.insert("vmsplice".to_string(), 75);
    map.insert("wait4".to_string(), 260);
    map.insert("waitid".to_string(), 95);
    map.insert("write".to_string(), 64);
    map.insert("writev".to_string(), 66);
}
