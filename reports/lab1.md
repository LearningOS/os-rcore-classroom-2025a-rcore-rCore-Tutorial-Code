# Lab1

## 实验题目

在 ch3 中，我们的系统已经能够支持多个任务分时轮流运行，我们希望引入一个新的系统调用 ``sys_trace``（ID 为 410）用来追踪当前任务系统调用的历史信息，并做对应的修改。定义如下。

```rust
fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize
```

- 调用规范：
    - 这个系统调用有三种功能，根据 trace_request 的值不同，执行不同的操作：

    - 如果 trace_request 为 0，则 id 应被视作 *const u8 ，表示读取当前任务 id 地址处一个字节的无符号整数值。此时应忽略 data 参数。返回值为 id 地址处的值。

    如果 trace_request 为 1，则 id 应被视作 *mut u8 ，表示写入 data （作为 u8，即只考虑最低位的一个字节）到该用户程序 id 地址处。返回值应为0。

    如果 trace_request 为 2，表示查询当前任务调用编号为 id 的系统调用的次数，返回值为这个调用次数。本次调用也计入统计 。

    否则，忽略其他参数，返回值为 -1。

- 说明：
    - 你可能会注意到，这个调用的读写并不安全，使用不当可能导致崩溃。这是因为在下一章节实现地址空间之前，系统中缺乏隔离机制。所以我们 不要求你实现安全检查机制，只需通过测试用例即可 。

    - 你还可能注意到，这个系统调用读写本任务内存的功能并不是很有用。这是因为作业的灵感来源 syscall 主要依靠 trace 功能追踪其他任务的信息，但在本章节我们还没有进程、线程等概念，所以简化了操作，只要求追踪自身的信息。

## 解决方案

首先，我们需要在 `syscall` 中添加 `sys_trace` 系统调用的处理函数。

```rust
// TODO: implement the syscall
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            let id = _id as *const u8;
            unsafe { *id as isize }
        },
        1 => {
            let id = _id as *mut u8;
            unsafe { *id = _data as u8 };
            0
        },
        2 => {
            get_syscall_count(_id) as isize
        },
        _ => {
            -1
        }
    }
}
```

- 0和1的情况比较简单，直接根据参数进行读写即可。

- 2的情况比较复杂，我们需要根据参数 id 来查询当前任务调用编号为 id 的系统调用的次数。

根据测试用例的要求，每个app的系统调用次数需要单独存储，因此不能直接使用一个内核中的数组来存储系统调用的次数

应当在每个TaskControlBlock中添加对系统调用次数的记录

```rust
/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The syscall count
    pub syscall_counts: [usize; MAX_SYSCALL_ID],
}
```

然后由 `TaskManager` 来对外暴露获取与增加系统调用计数的接口

```rust
/// Get syscall count of current task
fn get_syscall_count(&self, syscall_id: usize) -> usize {
    let inner = self.inner.exclusive_access();
    let current = inner.current_task;
    let tcb = &inner.tasks[current];
    let count = tcb.syscall_counts[syscall_id];
    drop(inner);
    count
}

/// Add syscall count of current task
fn add_syscall_count(&self, syscall_id: usize) {
    // println!("add syscall count, syscall_id = {}", syscall_id);
    let mut inner = self.inner.exclusive_access();
    let current = inner.current_task;
    let tcb = &mut inner.tasks[current];
    tcb.syscall_counts[syscall_id] += 1;
    drop(inner);
}
```

最后，在 `syscall` 中调用 `TaskManager` 提供的接口来增加系统调用计数

```rust
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    add_syscall_count(syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
```

```rust
// TODO: implement the syscall
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let id = id as *const u8;
            unsafe { *id as isize }
        },
        1 => {
            let id = id as *mut u8;
            unsafe { *id = data as u8 };
            0
        },
        2 => {
            let count = get_syscall_count(id);
            count as isize
        },
        _ => {
            -1
        }
    }
}
```
