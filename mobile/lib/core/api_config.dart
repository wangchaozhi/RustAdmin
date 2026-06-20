import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// 后端地址配置：支持用户在设置里自定义 IP/端口，持久化到设备。
/// 优先级：用户自定义 > 编译参数 API_BASE_URL > 平台默认。
class ApiConfig {
  static const String _key = 'api_base_url_override';
  static const String _definedBaseUrl = String.fromEnvironment('API_BASE_URL');

  static String? _override;

  /// 应在 runApp 之前调用，加载已保存的自定义地址。
  static Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    final value = prefs.getString(_key);
    _override = (value != null && value.isNotEmpty) ? value : null;
  }

  /// 用户自定义地址，null 表示使用默认。
  static String? get override => _override;

  /// 是否正在使用自定义地址。
  static bool get isOverridden => _override != null && _override!.isNotEmpty;

  /// 实际请求使用的 baseUrl。
  static String get baseUrl {
    if (isOverridden) return _override!;
    if (_definedBaseUrl.isNotEmpty) return _definedBaseUrl;
    return platformDefault;
  }

  /// 平台默认地址：Android 模拟器走 10.0.2.2，其余走 127.0.0.1。
  static String get platformDefault {
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      return 'http://10.0.2.2:8080';
    }
    return 'http://127.0.0.1:8080';
  }

  /// 保存自定义地址；传空字符串等同恢复默认。
  static Future<void> save(String url) async {
    final prefs = await SharedPreferences.getInstance();
    final value = url.trim();
    if (value.isEmpty) {
      await prefs.remove(_key);
      _override = null;
    } else {
      await prefs.setString(_key, value);
      _override = value;
    }
  }

  /// 恢复默认地址。
  static Future<void> reset() => save('');

  /// 由 IP/域名 + 端口拼出 http 地址，自动剥离用户误输入的协议头和多余斜杠。
  static String buildUrl({required String host, required String port}) {
    var cleanHost = host.trim();
    cleanHost = cleanHost.replaceFirst(RegExp(r'^https?://'), '');
    cleanHost = cleanHost.replaceAll(RegExp(r'/+$'), '');
    final cleanPort = port.trim();
    if (cleanPort.isEmpty) return 'http://$cleanHost';
    return 'http://$cleanHost:$cleanPort';
  }
}
