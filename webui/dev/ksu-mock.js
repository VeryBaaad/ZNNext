/*
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
  if (locale === 'en' || locale === 'zh-CN') {
    try {
      localStorage.setItem('znn_locale', locale);
    } catch (error) {}
  }

  var ui = params.get('ui');
  if (ui === 'material' || ui === 'miuix' || ui === 'zn') {
    try {
      localStorage.setItem('znn_ui', ui);
    } catch (error) {}
  }

  if (params.get('ksu') === '0') return;

  var latency = Number(params.get('slow') || 120);
  var failures = (params.get('fail') || '').split(',').filter(Boolean);
  var pid = function () {
    return 300 + Math.floor(Math.random() * 9000);
  };

  var CONFIG_KEYS = {
    inline: 'inlineHook',
    plt: 'pltHook',
    mode: 'mode',
    debugdata: 'debugdata',
    'disable-zn': 'disableZn',
  };

  var config = {
    inlineHook: { value: 'dobby', options: ['dobby', 'shadowhook', 'and64inlinehook', 'frida-gum', 'rv64hook', 'hookzz', 'disable'] },
    pltHook: { value: 'lsplt', options: ['lsplt', 'plti', 'bytehook', 'xhook', 'plthook', 'disable'] },
    mode: { value: 'proc', options: ['auto', 'kernel', 'ptrace', 'proc', 'late'] },
    debugdata: { value: 'lzma', options: ['lzma', 'xz'] },
    disableZn: { value: 'true', options: ['true', 'false'] },
  };

  var modules = [
    {
      id: 'zygisknextsu',
      name: 'ZNNext',
      version: 'v0.0.0',
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
        { name: 'artd', reason: 'test' },
        { name: 'hyos_spawner', reason: 'test' },
      ],
    },
  ];

  if (params.get('modules') === 'none') {
    modules = [];
  }

  var state = {
    running: params.get('status') !== 'inactive',
    mode: 'proc',
    config: config,
    modules: modules,
  };

  function snapshot(cmd, operands) {
    switch (cmd) {
      case 'status':
        return { running: state.running, pid: state.running ? 4242 : 0, mode: state.mode };
      case 'system':
        return {
          kernel: '5.15.148-android13-8-g2b4c1f0a1b2c-ab1234567',
          sdk: 34,
          abi: 'arm64-v8a',
          abilist: 'arm64-v8a,armeabi-v7a,armeabi',
          root: { magisk: null, kernelSU: 'ksud v3.0.0', apatch: null },
        };
      case 'modules':
        return state.modules;
      case 'config':
        return state.config;
      case 'config-set': {
        var kind = operands[0];
        var value = operands[1];
        var key = CONFIG_KEYS[kind];
        var entry = key ? state.config[key] : null;
        if (!entry) {
          throw new Error(
            'usage: injector --ctl config-set <inline|plt|mode|debugdata|disable-zn> <value>',
          );
        }
        if (entry.options.indexOf(value) < 0) {
          throw new Error('invalid ' + kind + ' value "' + value + '" for this device');
        }
        entry.value = value;
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
        if (failures.indexOf(cmd) >= 0) {
          window[callbackName](1, '', 'stub: forced failure for "' + cmd + '"');
          return;
        }
        try {
          window[callbackName](0, JSON.stringify(snapshot(cmd, operands)), '');
        } catch (error) {
          window[callbackName](1, '', String(error && error.message ? error.message : error));
        }
      }, latency);
    },
    moduleInfo: function () {
      return JSON.stringify({ moduleDir: '/data/adb/modules/zygisknextsu' });
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
    '[znn preview] KernelSU bridge stubbed. Overrides: ?ksu=0 ?status=inactive ?modules=none ' +
      '?ui=material|miuix|zn ?theme=light|dark|auto ?locale=zh-CN|en ?slow=MS ?fail=config,modules',
  );
})();
