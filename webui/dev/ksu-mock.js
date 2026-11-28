/*
 * SPDX-License-Identifier: GPL-3.0-or-later AND LicenseRef-DCLPL-2.0
 *
 * This file is part of ZNNext.
 *
 * ZNNext is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ZNNext is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ZNNext. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

(function () {
  if (window.ksu) return;

  var params = new URLSearchParams(location.search);

  var theme = params.get('theme');
  if (theme === 'light' || theme === 'dark' || theme === 'auto') {
    try {
      localStorage.setItem('znn_theme', theme);
    } catch (error) {}
  }

  var locale = params.get('locale');
  if (locale === 'en' || locale === 'zh-CN' || locale === 'lzh' || locale === 'ru') {
    try {
      localStorage.setItem('znn_locale', locale);
    } catch (error) {}
  }

  var ui = params.get('ui');
  if (ui === 'material' || ui === 'miuix' || ui === 'zn' || ui === 'ios') {
    try {
      localStorage.setItem('znn_ui', ui);
    } catch (error) {}
  }

  var seed = params.get('seed');
  if (seed) {
    var seedMatch = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(seed);
    if (seedMatch) {
      var digits = seedMatch[1].toLowerCase();
      if (digits.length === 3) {
        digits =
          digits[0] + digits[0] +
          digits[1] + digits[1] +
          digits[2] + digits[2];
      }
      try {
        localStorage.setItem('znn_material_seed', '#' + digits);
      } catch (error) {}
    }
  }

  var spec = (params.get('spec') || '').toUpperCase().replace('-', '_');
  if (spec === '2021') spec = 'SPEC_2021';
  if (spec === '2025') spec = 'SPEC_2025';
  if (spec === 'SPEC_2021' || spec === 'SPEC_2025') {
    try {
      localStorage.setItem('znn_material_spec', spec);
    } catch (error) {}
  }

  if (params.get('ksu') === '0') return;

  var latency = Number(params.get('slow') || 120);
  var failures = (params.get('fail') || '').split(',').filter(Boolean);
  var readCounts = { status: 0, system: 0, modules: 0, config: 0, logs: 0, bugreport: 0 };
  var moduleDir = '/data/adb/modules/zygisknextsu';
  var logcatCommand = "set -o pipefail; /system/bin/logcat -b main -d -v brief znndaemon:I '*:S' | /system/bin/tail -n 100";
  var daemonLogs = [
    '--------- beginning of main',
    'E/znndaemon( 4242): module verification failed: signature mismatch at /data/adb/modules/example/mazoku',
    'E/znndaemon( 4242): cannot serve daemon state: permission denied in /data/adb/zygisknextsu',
    'I/znndaemon( 4242): preview-only unknown entry from /data/user/0/com.example.private/files/session.txt',
  ].join('\n');
  var pid = function () {
    return 300 + Math.floor(Math.random() * 9000);
  };

  var CONFIG_KEYS = {
    inline: 'inlineHook',
    inline32: 'inlineHook32',
    plt: 'pltHook',
    plt32: 'pltHook32',
    mode: 'mode',
    debugdata: 'debugdata',
    'disable-zn': 'disableZn',
    'hot-update': 'hotUpdate',
  };
  var configuredKeys = new Set();
  var previousConfig = null;

  var config = {
    inlineHook: { value: 'dobby', options: ['dobby', 'shadowhook', 'and64inlinehook', 'frida-gum', 'rv64hook', 'hookzz', 'urhook', 'whale', 'baaadhook', 'yukiihook', 'sandhook', 'srxinlinehook', 'disable'] },
    inlineHook32: { value: 'dobby', options: ['dobby', 'shadowhook', 'frida-gum', 'hookzz', 'whale', 'baaadhook', 'sandhook', 'disable'] },
    pltHook: { value: 'lsplt', options: ['lsplt', 'plti', 'bytehook', 'xhook', 'plthook', 'urhook', 'baaadhook', 'yukiplt', 'profilo', 'srxhook', 'disable'] },
    pltHook32: { value: 'lsplt', options: ['lsplt', 'plti', 'bytehook', 'xhook', 'plthook', 'baaadhook', 'yukiplt', 'profilo', 'disable'] },
    mode: { value: 'proc', options: ['auto', 'kernel', 'ptrace', 'proc', 'late'] },
    debugdata: { value: 'lzma', options: ['lzma', 'xz', 'libarchive'] },
    disableZn: { value: 'true', options: ['true', 'false'] },
    hotUpdate: { value: 'false', options: ['true', 'false'] },
  };

  var modules = [
    {
      id: 'zygisknextsu',
      name: 'ZNNext',
      version: 'v2.22.0.1',
      processes: [
        { pid: pid(), name: 'adbd' },
        { pid: pid(), name: 'netd' },
        { pid: pid(), name: 'artd' },
      ],
      failed: [],
    },
    {
      id: 'zygisk_lsposed',
      name: 'LSPosed',
      version: 'v1.9.2',
      processes: [
        { pid: pid(), name: 'artd' },
        { pid: pid(), name: 'hyos_spawner' },
      ],
      failed: [
        {
          name: '/apex/com.android.art/bin/artd',
          reason: 'memfd_create failed while preparing a module library',
          pid: pid(),
          lastSeenMs: Date.now(),
        },
        { name: 'hyos_spawner', reason: 'no injection window' },
      ],
    },
  ];

  var modulesOverride = params.get('modules');
  var disabledModule = {
    id: 'disabled_example',
    name: 'Disabled Example',
    version: 'v1.4.0',
    disabled: true,
    processes: [],
    failed: [],
  };
  var undeclaredModule = {
    id: 'undeclared_example',
    name: 'Undeclared Example',
    version: 'v1.6.0',
    disabled: false,
    declarationIssue: 'noDeclarations',
    processes: [],
    failed: [],
  };
  var missingLibrariesModule = {
    id: 'missing_libraries_example',
    name: 'Missing Libraries Example',
    version: 'v1.7.0',
    disabled: false,
    declarationIssue: 'missingLibraries',
    missingLibraries: [
      'lib/arm64-v8a/libexample.so',
      'lib/arm64-v8a/plugins/feature-set/release/libexample_module_with_optional_native_support_and_diagnostics.so',
    ],
    processes: [],
    failed: [],
  };
  var waitingModule = {
    id: 'waiting_example',
    name: 'Waiting Example',
    version: 'v1.0.0',
    waitingForTarget: true,
    processes: [],
    failed: [],
  };
  if (modulesOverride === 'none') {
    modules = [];
  } else if (modulesOverride === 'no-version') {
    delete modules[0].version;
  } else if (modulesOverride === 'disabled') {
    modules = [disabledModule];
  } else if (modulesOverride === 'no-declarations') {
    modules = [undeclaredModule];
  } else if (modulesOverride === 'missing-libraries') {
    modules = [missingLibrariesModule];
  } else if (modulesOverride === 'waiting') {
    modules = [waitingModule];
  } else if (modulesOverride === 'mixed') {
    modules = [
      disabledModule,
      undeclaredModule,
      {
        ...missingLibrariesModule,
        processes: [{ pid: 2215, name: 'native_service' }],
      },
      waitingModule,
      {
        id: 'idle_example',
        name: 'Idle Example',
        version: 'v3.0.0',
        disabled: false,
        processes: [],
        failed: [],
      },
      { ...modules[1], disabled: true },
      modules[0],
    ];
  }

  var statusOverride = params.get('status');
  var state = {
    running: statusOverride !== 'inactive',
    mode: 'proc',
    config: config,
    modules: modules,
  };

  function isOptionalConfigKey(kind) {
    return kind === 'inline32' || kind === 'plt32';
  }

  function setConfigValue(kind, value) {
    if (value === null) {
      configuredKeys.delete(kind);
    } else {
      state.config[CONFIG_KEYS[kind]].value = value;
      configuredKeys.add(kind);
    }
    var primary = isOptionalConfigKey(kind) ? kind.slice(0, -2) : kind;
    var secondary = primary + '32';
    if (isOptionalConfigKey(secondary) && !configuredKeys.has(secondary)) {
      var engine = state.config[CONFIG_KEYS[primary]].value;
      var entry32 = state.config[CONFIG_KEYS[secondary]];
      entry32.value = entry32.options.indexOf(engine) >= 0 ? engine : entry32.options[0];
    }
    if (kind !== 'mode' || !state.running) return;
    switch (value) {
      case 'proc':
      case 'late':
        state.mode = value;
        break;
      case 'auto':
      case 'kernel':
        state.mode = 'kernel';
        break;
      case 'ptrace':
        if (state.mode === 'kernel') state.mode = 'proc';
        break;
    }
  }

  function exportConfig() {
    var exported = {};
    Object.keys(CONFIG_KEYS).forEach(function (kind) {
      if (isOptionalConfigKey(kind) && !configuredKeys.has(kind)) return;
      exported[kind] = state.config[CONFIG_KEYS[kind]].value;
    });
    return exported;
  }

  function diffConfig(cmd, operands) {
    if (operands.length !== 1) throw new Error('usage: znnd --ctl ' + cmd + ' <json>');
    var requested = JSON.parse(operands[0]);
    if (typeof requested !== 'object' || requested === null || Array.isArray(requested)) {
      throw new Error(cmd + ' requires a JSON object');
    }
    var requestedKeys = Object.keys(requested);
    requestedKeys.forEach(function (kind) {
      if (!Object.prototype.hasOwnProperty.call(CONFIG_KEYS, kind)) {
        throw new Error('unknown configuration key: ' + kind);
      }
    });
    var diff = compareConfig(requested);
    var keyCount = 0;
    for (var token of operands[0].matchAll(/"(?:[^"\\]|\\.)*"(\s*:)?/g)) {
      if (token[1]) keyCount += 1;
    }
    if (keyCount !== requestedKeys.length) {
      throw new Error('duplicate configuration key');
    }
    return diff;
  }

  function compareConfig(requested) {
    var current = exportConfig();
    var changes = {};
    Object.keys(CONFIG_KEYS).forEach(function (kind) {
      var target = requested[kind];
      if (isOptionalConfigKey(kind) && target == null) {
        target = null;
      } else if (target === undefined) {
        throw new Error('missing configuration key: ' + kind);
      } else if (typeof target !== 'string' || state.config[CONFIG_KEYS[kind]].options.indexOf(target) < 0) {
        throw new Error('invalid ' + kind + ' value for this device');
      }
      var previous = current[kind] ?? null;
      if (previous !== target) changes[kind] = { from: previous, to: target };
    });
    return { changes: changes };
  }

  function applyConfigChanges(diff, remember) {
    var changedKeys = Object.keys(diff.changes);
    if (remember && changedKeys.length) previousConfig = exportConfig();
    changedKeys.forEach(function (kind) {
      setConfigValue(kind, diff.changes[kind].to);
    });
    return diff;
  }

  function snapshot(cmd, operands) {
    switch (cmd) {
      case 'logs':
        return params.get('logs') === 'empty' ? '' : daemonLogs + '\n';
      case 'bugreport':
        if (!operands[0] || operands.length > 2) throw new Error('invalid bug report arguments');
        return 'mock://Download/ZNNext-bugreport-' + Date.now() + '-' + readCounts.bugreport + '.tar.gz\n';
      case 'status': {
        var status = {
          running: state.running,
          pid: state.running ? 4242 : 0,
          mode: state.running ? state.mode : 'unknown',
        };
        if (state.running && statusOverride !== 'legacy') {
          status.version = 'v2.22.0.1-128-a1b2c3d-release';
        }
        return status;
      }
      case 'system':
        if (!state.running) {
          return {
            kernel: '',
            sdk: 0,
            abi: '',
            abilist: '',
            root: { magisk: '', kernelSU: '', apatch: '' },
          };
        }
        return {
          kernel: '5.15.148-android13-8-g2b4c1f0a1b2c-ab1234567',
          sdk: 34,
          pageSize: statusOverride === 'legacy' ? undefined : Number(params.get('pagesize') ?? 4096),
          abi: 'arm64-v8a',
          abilist: 'arm64-v8a,armeabi-v7a,armeabi',
          root: { magisk: null, kernelSU: 'ksud v3.0.0', apatch: null },
        };
      case 'modules':
        return state.running ? state.modules : [];
      case 'config':
        return state.config;
      case 'config-export':
        if (operands.length !== 0) {
          throw new Error('usage: znnd --ctl config-export');
        }
        return exportConfig();
      case 'config-diff':
        return diffConfig(cmd, operands);
      case 'config-import':
        return applyConfigChanges(diffConfig(cmd, operands), true);
      case 'config-reset':
        if (operands.length !== 0) throw new Error('usage: znnd --ctl config-reset');
        return applyConfigChanges(compareConfig({
          inline: state.config.inlineHook.options[0],
          plt: state.config.pltHook.options[0],
          mode: 'auto',
          debugdata: state.config.debugdata.options[0],
          'disable-zn': 'true',
          'hot-update': 'false',
        }), true);
      case 'config-undo': {
        if (operands.length !== 0) throw new Error('usage: znnd --ctl config-undo');
        if (previousConfig === null) throw new Error('no configuration change to undo');
        var undone = applyConfigChanges(compareConfig(previousConfig), false);
        previousConfig = null;
        return undone;
      }
      case 'config-set': {
        var kind = operands[0];
        var value = operands[1];
        var key = CONFIG_KEYS[kind];
        var entry = key ? state.config[key] : null;
        if (!entry) {
          throw new Error(
            'usage: znnd --ctl config-set <inline|inline32|plt|plt32|mode|debugdata|disable-zn|hot-update> <value>',
          );
        }
        if (entry.options.indexOf(value) < 0) {
          throw new Error('invalid ' + kind + ' value "' + value + '" for this device');
        }
        var before = exportConfig();
        if (before[kind] !== value) previousConfig = before;
        setConfigValue(kind, value);
        return state.config;
      }
      default:
        throw new Error('unknown command: ' + cmd);
    }
  }

  function splitCommand(command) {
    var tokens = [];
    var current = '';
    var quoted = false;
    var started = false;
    for (var i = 0; i < command.length; i++) {
      var ch = command.charAt(i);
      if (ch === '\\' && !quoted && i + 1 < command.length) {
        current += command.charAt(++i);
        started = true;
      } else if (ch === "'") {
        quoted = !quoted;
        started = true;
      } else if (!quoted && /\s/.test(ch)) {
        if (started) {
          tokens.push(current);
          current = '';
          started = false;
        }
      } else {
        current += ch;
        started = true;
      }
    }
    if (started) tokens.push(current);
    return tokens;
  }

  window.ksu = {
    exec: function (command, options, callbackName) {
      window.setTimeout(function () {
        var argc = splitCommand(command);
        var ctl = argc.indexOf('--ctl');
        var cmd = ctl >= 0 ? argc[ctl + 1] : '';
        var operands = ctl >= 0 ? argc.slice(ctl + 2) : [];
        if (command === logcatCommand) {
          cmd = 'logs';
        } else if (argc[0] === '/system/bin/sh' && argc[1] === moduleDir + '/bugreport.sh') {
          cmd = 'bugreport';
          operands = argc.slice(2);
        }
        var scheduledFailure = false;
        if (Object.prototype.hasOwnProperty.call(readCounts, cmd)) {
          readCounts[cmd] += 1;
          scheduledFailure = failures.indexOf(cmd + '@' + readCounts[cmd]) >= 0;
        }
        if (failures.indexOf(cmd) >= 0 || scheduledFailure) {
          window[callbackName](1, '', 'stub: forced failure for "' + cmd + '"');
          return;
        }
        try {
          var output = snapshot(cmd, operands);
          window[callbackName](0, typeof output === 'string' ? output : JSON.stringify(output), '');
        } catch (error) {
          window[callbackName](1, '', String(error && error.message ? error.message : error));
        }
      }, latency);
    },
    moduleInfo: function () {
      return JSON.stringify({ moduleDir: moduleDir });
    },
    toast: function (message) {
      console.info('[znn preview] toast:', message);
    },
    fullScreen: function () {},
    enableEdgeToEdge: function () {},
    listPackages: function () {
      return '[]';
    },
  };

  console.info(
    '[znn preview] KernelSU bridge stubbed. Overrides: ?ksu=0 ?status=inactive|legacy ?pagesize=16384 ?modules=none|no-version|disabled|no-declarations|missing-libraries|waiting|mixed ?logs=empty ' +
      '?ui=material|miuix|zn|ios ?theme=light|dark|auto ?locale=zh-CN|lzh|en|ru ?slow=MS ?fail=logs@1,bugreport@1',
  );
})();
