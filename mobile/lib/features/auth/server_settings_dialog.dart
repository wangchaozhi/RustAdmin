import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../core/api_config.dart';

/// 弹出服务器设置对话框，返回 true 表示地址有改动。
Future<bool?> showServerSettingsDialog(BuildContext context) {
  return showDialog<bool>(
    context: context,
    builder: (_) => const _ServerSettingsDialog(),
  );
}

class _ServerSettingsDialog extends StatefulWidget {
  const _ServerSettingsDialog();

  @override
  State<_ServerSettingsDialog> createState() => _ServerSettingsDialogState();
}

class _ServerSettingsDialogState extends State<_ServerSettingsDialog> {
  late final TextEditingController _hostController;
  late final TextEditingController _portController;
  String _error = '';

  @override
  void initState() {
    super.initState();
    final uri = Uri.tryParse(ApiConfig.baseUrl);
    _hostController = TextEditingController(text: uri?.host ?? '');
    _portController = TextEditingController(
      text: (uri != null && uri.hasPort) ? '${uri.port}' : '',
    );
  }

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final host = _hostController.text.trim().replaceFirst(
      RegExp(r'^https?://'),
      '',
    );
    final port = _portController.text.trim();
    if (host.isEmpty) {
      setState(() => _error = '请输入服务器 IP 或域名');
      return;
    }
    if (port.isNotEmpty) {
      final value = int.tryParse(port);
      if (value == null || value < 1 || value > 65535) {
        setState(() => _error = '端口需为 1-65535 的数字');
        return;
      }
    }
    await ApiConfig.save(ApiConfig.buildUrl(host: host, port: port));
    if (!mounted) return;
    Navigator.pop(context, true);
  }

  Future<void> _reset() async {
    await ApiConfig.reset();
    if (!mounted) return;
    Navigator.pop(context, true);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(20)),
      title: const Text('服务器设置'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '当前地址：${ApiConfig.baseUrl}',
            style: const TextStyle(color: Color(0xFF6B7280), fontSize: 12),
          ),
          const SizedBox(height: 14),
          TextField(
            controller: _hostController,
            autofocus: true,
            keyboardType: TextInputType.url,
            decoration: const InputDecoration(
              labelText: '服务器 IP / 域名',
              hintText: '例如 192.168.1.23',
              prefixIcon: Icon(Icons.dns_rounded),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _portController,
            keyboardType: TextInputType.number,
            inputFormatters: [FilteringTextInputFormatter.digitsOnly],
            decoration: const InputDecoration(
              labelText: '端口',
              hintText: '例如 8080',
              prefixIcon: Icon(Icons.numbers_rounded),
            ),
          ),
          if (_error.isNotEmpty) ...[
            const SizedBox(height: 10),
            Text(
              _error,
              style: const TextStyle(color: Color(0xFFDC2626), fontSize: 13),
            ),
          ],
          const SizedBox(height: 8),
          const Text(
            '默认使用 http 协议。修改后下次登录与数据请求即生效。',
            style: TextStyle(color: Color(0xFF9CA3AF), fontSize: 11),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: _reset,
          child: const Text('恢复默认'),
        ),
        TextButton(
          onPressed: () => Navigator.pop(context, false),
          child: const Text('取消'),
        ),
        FilledButton(
          style: FilledButton.styleFrom(
            backgroundColor: const Color(0xFF166534),
          ),
          onPressed: _save,
          child: const Text('保存'),
        ),
      ],
    );
  }
}
