use super::MAX_SYSCALLS;
use super::SYSCALL_COUNTS;

pub fn increment_syscall(syscall_num: usize) {
    let mut counts = SYSCALL_COUNTS.lock();
    if syscall_num < MAX_SYSCALLS {
        counts[syscall_num] += 1;
    }
}

pub fn get_count(syscall_num: usize) -> usize {
    let counts = SYSCALL_COUNTS.lock();
    if syscall_num < MAX_SYSCALLS {
        counts[syscall_num]
    } else {
        0
    }
}

pub fn clear_counts() {
    let mut counts = SYSCALL_COUNTS.lock();
    for i in 0..MAX_SYSCALLS {
        counts[i] = 0;
    }
}