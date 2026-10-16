/*
 * This file is part of Zygisk Next Next.
 *
 * Zygisk Next Next is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Zygisk Next Next is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Zygisk Next Next. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

#include <compiler.h>
#include <kpmodule.h>
#include <hook.h>
#include <syscall.h>
#include <kputils.h>
#include <kbtf.h>
#include <asm/current.h>
#include <linux/printk.h>
#include <uapi/asm-generic/unistd.h>

#include "znn_format.h"
#include "znn.h"

#ifndef ZNN_KPM_VERSION
#define ZNN_KPM_VERSION "0.0.0-unknown"
#endif

_Static_assert(ZNN_PRCTL_NR == __NR_prctl, "prctl syscall number changed");

KPM_NAME(ZNN_NAME);
KPM_VERSION(ZNN_KPM_VERSION);
KPM_LICENSE("GPL-3.0-or-later");
KPM_AUTHOR("VeryBaaad");
KPM_DESCRIPTION("Optional Zygisk Next Next exec observation accelerator (experimental)");

#define ZNN_PIDTYPE_PID 0
#define ZNN_PIDTYPE_TGID 1

struct znn_exec_record {
    uint64_t seq;
    int32_t pid;
    int32_t tgid;
    int32_t ppid;
    int32_t reserved;
};

static struct znn_exec_record znn_ring[ZNN_RING_CAPACITY];
static uint64_t znn_ring_written;
static int znn_execve_hooked;
static int znn_execveat_hooked;
static int znn_channel_hooked;

static int32_t (*znn_task_pid_nr_ns)(void *task, int type, void *ns);
static long znn_real_parent_offset = -1;
static long znn_task_cred_offset = -1;
static long znn_cred_uid_offset = -1;

static int32_t znn_task_nr(void *task, int type)
{
    if (!znn_task_pid_nr_ns || !task) return -1;
    return (int32_t)znn_task_pid_nr_ns(task, type, 0);
}

static int32_t znn_parent_nr(void *task)
{
    if (znn_real_parent_offset <= 0 || !task) return -1;
    return znn_task_nr(*(void **)((char *)task + znn_real_parent_offset), ZNN_PIDTYPE_PID);
}

static void znn_note_exec(void)
{
    void *task = current;
    uint64_t seq = __atomic_add_fetch(&znn_ring_written, 1, __ATOMIC_RELAXED);
    struct znn_exec_record *record = &znn_ring[(seq - 1) % ZNN_RING_CAPACITY];

    record->seq = seq;
    record->pid = znn_task_nr(task, ZNN_PIDTYPE_PID);
    record->tgid = znn_task_nr(task, ZNN_PIDTYPE_TGID);
    record->ppid = znn_parent_nr(task);
    record->reserved = 0;
}

static void znn_before_execve(hook_fargs3_t *args, void *udata)
{
    (void)args;
    (void)udata;
    znn_note_exec();
}

static void znn_before_execveat(hook_fargs5_t *args, void *udata)
{
    (void)args;
    (void)udata;
    znn_note_exec();
}

static int znn_caller_uid_is_root(void)
{
    if (znn_task_cred_offset < 0 || znn_cred_uid_offset < 0) return 1;

    void *task = current;
    if (!task) return 1;
    void *cred = *(void **)((char *)task + znn_task_cred_offset);
    if (!cred) return 1;
    return *(uint32_t *)((char *)cred + znn_cred_uid_offset) == 0;
}

static long znn_channel_write_records(char __user *user_buffer, uint64_t buffer_length, uint64_t since_seq)
{
    uint64_t written = __atomic_load_n(&znn_ring_written, __ATOMIC_ACQUIRE);
    if (since_seq >= written) return 0;

    uint64_t available = written < ZNN_RING_CAPACITY ? written : ZNN_RING_CAPACITY;
    uint64_t oldest = written - available + 1;
    uint64_t first = since_seq + 1 > oldest ? since_seq + 1 : oldest;
    uint64_t count = written - first + 1;

    long total = 0;
    for (uint64_t i = 0; i < count; i++) {
        uint64_t seq = first + i;
        const struct znn_exec_record *record = &znn_ring[(seq - 1) % ZNN_RING_CAPACITY];
        char line[ZNN_LINE_CAPACITY];
        int length = znn_format_exec_record(line, (int)sizeof(line), seq, record->pid, record->tgid, record->ppid);
        if (length <= 0) break;
        if ((uint64_t)total + (uint64_t)length > buffer_length) break;
        if (compat_copy_to_user(user_buffer + total, line, length) != length) return -EFAULT;
        total += length;
    }
    return total;
}

static void znn_channel_before(hook_fargs5_t *args, void *udata)
{
    (void)udata;

    if (syscall_argn(args, 0) != ZNN_KPM_MAGIC) return;

    args->skip_origin = 1;

    uint64_t user_buffer = syscall_argn(args, 1);
    uint64_t buffer_length = syscall_argn(args, 2);
    uint64_t since_seq = syscall_argn(args, 3);

    if (buffer_length < 1 || buffer_length > ZNN_CHANNEL_MAX_LENGTH) {
        args->ret = (uint64_t)(long)-EINVAL;
        return;
    }

    if (!znn_caller_uid_is_root()) {
        args->ret = (uint64_t)(long)-EPERM;
        return;
    }

    args->ret = (uint64_t)znn_channel_write_records((char __user *)(uintptr_t)user_buffer, buffer_length, since_seq);
}

static void znn_unhook_all(void)
{
    if (znn_channel_hooked) {
        unhook_syscalln(__NR_prctl, znn_channel_before, 0);
        znn_channel_hooked = 0;
    }
    if (znn_execveat_hooked) {
        unhook_syscalln(__NR_execveat, znn_before_execveat, 0);
        znn_execveat_hooked = 0;
    }
    if (znn_execve_hooked) {
        unhook_syscalln(__NR_execve, znn_before_execve, 0);
        znn_execve_hooked = 0;
    }
}

static int znn_parse_read_command(const char *args, uint32_t *count)
{
    if (!args) return -1;
    while (*args == ' ' || *args == '\t') args++;
    if (args[0] != 'r' || args[1] != 'e' || args[2] != 'a' || args[3] != 'd') return -1;
    args += 4;
    if (*args != '\0' && *args != ' ' && *args != '\t') return -1;
    while (*args == ' ' || *args == '\t') args++;

    uint32_t value = 0;
    int digits = 0;
    while (*args >= '0' && *args <= '9') {
        value = value * 10 + (uint32_t)(*args - '0');
        if (value > ZNN_RING_CAPACITY) value = ZNN_RING_CAPACITY;
        args++;
        digits++;
    }
    if (*args != '\0' && *args != ' ' && *args != '\t') return -1;
    *count = digits ? value : 0;
    return 0;
}

static long znn_control(const char *ctl_args, char *__user out_msg, int outlen)
{
    uint32_t requested = 0;
    if (!out_msg || outlen <= 0) return -1;
    if (znn_parse_read_command(ctl_args, &requested)) return -1;

    uint64_t written = __atomic_load_n(&znn_ring_written, __ATOMIC_ACQUIRE);
    uint64_t available = written < ZNN_RING_CAPACITY ? written : ZNN_RING_CAPACITY;
    uint32_t count = (requested == 0 || requested > available) ? (uint32_t)available : requested;
    uint64_t first = written - count;
    int total = 0;

    for (uint32_t i = 0; i < count; i++) {
        const struct znn_exec_record *record = &znn_ring[(first + i) % ZNN_RING_CAPACITY];
        char line[ZNN_LINE_CAPACITY];
        int length = znn_format_exec_record(line, (int)sizeof(line), record->seq, record->pid, record->tgid,
                                            record->ppid);
        if (length <= 0) break;
        if (total + length > outlen) break;
        if (compat_copy_to_user(out_msg + total, line, length) != length) break;
        total += length;
    }
    return total;
}

static long znn_init(const char *args, const char *event, void *reserved)
{
    (void)args;
    (void)event;
    (void)reserved;

    znn_task_pid_nr_ns =
        (int32_t(*)(void *, int, void *))(uintptr_t)kallsyms_lookup_name("__task_pid_nr_ns");
    long parent_offset = kp_btf_member_offset("task_struct", "real_parent");
    if (parent_offset > 0) znn_real_parent_offset = parent_offset;

    long cred_offset = kp_btf_member_offset("task_struct", "cred");
    long cred_uid_offset = cred_offset >= 0 ? kp_btf_member_offset("cred", "uid") : -1;
    if (cred_uid_offset >= 0) {
        znn_task_cred_offset = cred_offset;
        znn_cred_uid_offset = cred_uid_offset;
    }

    hook_err_t error = hook_syscalln(__NR_execve, 3, znn_before_execve, 0, 0);
    if (error) {
        pr_err("znn: cannot hook execve: %d\n", error);
        znn_unhook_all();
        return error;
    }
    znn_execve_hooked = 1;

    error = hook_syscalln(__NR_execveat, 5, znn_before_execveat, 0, 0);
    if (error) {
        pr_err("znn: cannot hook execveat: %d\n", error);
        znn_unhook_all();
        return error;
    }
    znn_execveat_hooked = 1;

    hook_err_t channel_error = hook_syscalln_override(__NR_prctl, 5, znn_channel_before, 0, 0);
    if (channel_error) {
        pr_err("znn: cannot hook prctl channel: %d\n", channel_error);
    } else {
        znn_channel_hooked = 1;
    }

    if (znn_task_cred_offset < 0)
        pr_info("znn: cred offsets unresolved, uid gate disabled, serving every caller\n");

    pr_info("znn %s ready (real_parent offset %ld, pid source %s, uid gate %s, channel %s)\n", ZNN_KPM_VERSION,
            znn_real_parent_offset, znn_task_pid_nr_ns ? "__task_pid_nr_ns" : "unavailable",
            znn_task_cred_offset >= 0 ? "cred.uid" : "unavailable", znn_channel_hooked ? "prctl" : "unavailable");
    return 0;
}

static long znn_exit(void *reserved)
{
    (void)reserved;
    znn_unhook_all();
    return 0;
}

KPM_INIT(znn_init);
KPM_CTL0(znn_control);
KPM_EXIT(znn_exit);
