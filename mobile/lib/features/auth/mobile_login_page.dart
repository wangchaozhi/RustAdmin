import 'package:flutter/material.dart';

import '../../core/api_client.dart';
import '../../core/api_config.dart';
import 'login_storage.dart';
import 'server_settings_dialog.dart';
import 'widgets/login_header.dart';

class MobileLoginPage extends StatefulWidget {
  const MobileLoginPage({super.key});

  @override
  State<MobileLoginPage> createState() => _MobileLoginPageState();
}

class _MobileLoginPageState extends State<MobileLoginPage> {
  final _storage = LoginStorage();
  final _usernameController = TextEditingController(text: 'user');
  final _passwordController = TextEditingController(text: '123456');

  bool _loading = false;
  bool _remember = true;
  bool _ready = false;
  String _usernameError = '';
  String _passwordError = '';

  @override
  void initState() {
    super.initState();
    _loadSavedLogin();
  }

  @override
  void dispose() {
    _usernameController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  Future<void> _loadSavedLogin() async {
    final saved = await _storage.load();
    if (!mounted) return;

    _usernameController.text = saved.username;
    _passwordController.text = saved.password;
    setState(() {
      _remember = saved.remember;
      _ready = true;
    });
  }

  Future<void> _login() async {
    if (!_validate()) return;

    setState(() => _loading = true);
    try {
      final username = _usernameController.text.trim();
      final password = _passwordController.text;
      final resp = await ApiClient().post('/api/mobile/login', {
        'username': username,
        'password': password,
      });

      if (!mounted) return;
      final tokens = resp['tokens'];
      if (tokens is! Map<String, dynamic>) {
        _showMessage(resp['error']?.toString() ?? '登录失败');
        return;
      }

      final accessToken = tokens['access_token']?.toString() ?? '';
      final refreshToken = tokens['refresh_token']?.toString() ?? '';
      if (accessToken.isEmpty || refreshToken.isEmpty) {
        _showMessage('Login response is missing tokens');
        return;
      }

      await _storage.save(
        username: username,
        password: password,
        remember: _remember,
        accessToken: accessToken,
        refreshToken: refreshToken,
      );
      if (!mounted) return;
      Navigator.pushReplacementNamed(context, '/home');
    } catch (e) {
      if (!mounted) return;
      _showMessage('登录失败: $e');
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  bool _validate() {
    final usernameError = _usernameController.text.trim().isEmpty
        ? '请输入用户名'
        : '';
    final passwordError = _passwordController.text.isEmpty ? '请输入密码' : '';
    setState(() {
      _usernameError = usernameError;
      _passwordError = passwordError;
    });
    return usernameError.isEmpty && passwordError.isEmpty;
  }

  void _showMessage(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
    );
  }

  Future<void> _openServerSettings() async {
    final changed = await showServerSettingsDialog(context);
    if (!mounted || changed != true) return;
    setState(() {}); // 刷新，让设置按钮的副标题反映新地址
    _showMessage('服务器地址已更新为 ${ApiConfig.baseUrl}');
  }

  @override
  Widget build(BuildContext context) {
    if (!_ready) {
      return const Scaffold(
        body: DecoratedBox(
          decoration: BoxDecoration(
            gradient: LinearGradient(
              colors: [Color(0xFFF7FAFC), Color(0xFFEAF5F2)],
              begin: Alignment.topCenter,
              end: Alignment.bottomCenter,
            ),
          ),
          child: Center(child: CircularProgressIndicator()),
        ),
      );
    }

    return Scaffold(
      resizeToAvoidBottomInset: true,
      body: Stack(
        children: [
          const _LoginBackground(),
          SafeArea(
            child: LayoutBuilder(
              builder: (context, constraints) {
                return SingleChildScrollView(
                  padding: const EdgeInsets.fromLTRB(20, 54, 20, 24),
                  child: ConstrainedBox(
                    constraints: BoxConstraints(
                      minHeight: constraints.maxHeight - 78,
                    ),
                    child: Center(
                      child: ConstrainedBox(
                        constraints: const BoxConstraints(maxWidth: 430),
                        child: _LoginCard(
                          usernameController: _usernameController,
                          passwordController: _passwordController,
                          usernameError: _usernameError,
                          passwordError: _passwordError,
                          remember: _remember,
                          loading: _loading,
                          onRememberChanged: (value) =>
                              setState(() => _remember = value),
                          onLogin: _login,
                        ),
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
          SafeArea(
            child: Align(
              alignment: Alignment.topRight,
              child: Padding(
                padding: const EdgeInsets.only(top: 4, right: 8),
                child: IconButton(
                  tooltip: '服务器设置',
                  onPressed: _openServerSettings,
                  icon: const Icon(
                    Icons.settings_rounded,
                    color: Color(0xFF166534),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _LoginBackground extends StatelessWidget {
  const _LoginBackground();

  @override
  Widget build(BuildContext context) {
    return const DecoratedBox(
      decoration: BoxDecoration(color: Color(0xFFF6F9F7)),
      child: CustomPaint(
        painter: _LoginBackgroundPainter(),
        child: SizedBox.expand(),
      ),
    );
  }
}

class _LoginCard extends StatelessWidget {
  const _LoginCard({
    required this.usernameController,
    required this.passwordController,
    required this.usernameError,
    required this.passwordError,
    required this.remember,
    required this.loading,
    required this.onRememberChanged,
    required this.onLogin,
  });

  final TextEditingController usernameController;
  final TextEditingController passwordController;
  final String usernameError;
  final String passwordError;
  final bool remember;
  final bool loading;
  final ValueChanged<bool> onRememberChanged;
  final VoidCallback onLogin;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return DecoratedBox(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(28),
        boxShadow: const [
          BoxShadow(
            color: Color(0x1F0F172A),
            blurRadius: 34,
            offset: Offset(0, 20),
          ),
          BoxShadow(
            color: Color(0x0F047857),
            blurRadius: 18,
            offset: Offset(0, 8),
          ),
        ],
      ),
      child: Material(
        color: Colors.white.withValues(alpha: 0.96),
        borderRadius: BorderRadius.circular(28),
        clipBehavior: Clip.antiAlias,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              height: 7,
              decoration: const BoxDecoration(
                gradient: LinearGradient(
                  colors: [
                    Color(0xFF2563EB),
                    Color(0xFF0F766E),
                    Color(0xFFF59E0B),
                  ],
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(24, 26, 24, 24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const LoginHeader(),
                  const SizedBox(height: 26),
                  TextField(
                    controller: usernameController,
                    textInputAction: TextInputAction.next,
                    autofillHints: const [AutofillHints.username],
                    decoration: _fieldDecoration(
                      label: '用户名',
                      hint: '请输入用户名',
                      icon: Icons.person_outline_rounded,
                      error: usernameError,
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: passwordController,
                    obscureText: true,
                    textInputAction: TextInputAction.done,
                    autofillHints: const [AutofillHints.password],
                    onSubmitted: (_) {
                      if (!loading) onLogin();
                    },
                    decoration: _fieldDecoration(
                      label: '密码',
                      hint: '默认密码 123456',
                      icon: Icons.lock_outline_rounded,
                      error: passwordError,
                    ),
                  ),
                  const SizedBox(height: 16),
                  _RememberPanel(
                    value: remember,
                    enabled: !loading,
                    onChanged: onRememberChanged,
                  ),
                  const SizedBox(height: 20),
                  FilledButton.icon(
                    onPressed: loading ? null : onLogin,
                    icon: loading
                        ? SizedBox(
                            width: 18,
                            height: 18,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                              color: colorScheme.onPrimary,
                            ),
                          )
                        : const Icon(Icons.login_rounded),
                    label: Text(
                      loading ? '登录中...' : '登录',
                      style: const TextStyle(fontWeight: FontWeight.w700),
                    ),
                    style: FilledButton.styleFrom(
                      minimumSize: const Size.fromHeight(54),
                      backgroundColor: const Color(0xFF166534),
                      foregroundColor: Colors.white,
                      disabledBackgroundColor: const Color(0xFF9CA3AF),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(16),
                      ),
                      elevation: 0,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  InputDecoration _fieldDecoration({
    required String label,
    required String hint,
    required IconData icon,
    required String error,
  }) {
    return InputDecoration(
      labelText: label,
      hintText: hint,
      errorText: error.isEmpty ? null : error,
      prefixIcon: Padding(
        padding: const EdgeInsetsDirectional.only(start: 4),
        child: Icon(icon),
      ),
      prefixIconColor: const Color(0xFF2563EB),
      filled: true,
      fillColor: const Color(0xFFF8FAFC),
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 17),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(16),
        borderSide: const BorderSide(color: Color(0xFFDDE3EA)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(16),
        borderSide: const BorderSide(color: Color(0xFFDDE3EA)),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(16),
        borderSide: const BorderSide(color: Color(0xFF2563EB), width: 1.4),
      ),
      errorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(16),
        borderSide: const BorderSide(color: Color(0xFFDC2626)),
      ),
      focusedErrorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(16),
        borderSide: const BorderSide(color: Color(0xFFDC2626), width: 1.4),
      ),
    );
  }
}

class _RememberPanel extends StatelessWidget {
  const _RememberPanel({
    required this.value,
    required this.enabled,
    required this.onChanged,
  });

  final bool value;
  final bool enabled;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      borderRadius: BorderRadius.circular(16),
      onTap: enabled ? () => onChanged(!value) : null,
      child: Ink(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
        decoration: BoxDecoration(
          color: value ? const Color(0xFFEFF6FF) : const Color(0xFFF8FAFC),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(
            color: value ? const Color(0xFFBFDBFE) : const Color(0xFFE5E7EB),
          ),
        ),
        child: Row(
          children: [
            Icon(
              value ? Icons.verified_user_rounded : Icons.shield_outlined,
              color: value ? const Color(0xFF2563EB) : const Color(0xFF64748B),
            ),
            const SizedBox(width: 12),
            const Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    '记住密码',
                    style: TextStyle(
                      color: Color(0xFF111827),
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  SizedBox(height: 2),
                  Text(
                    '下次打开自动回填账号和密码',
                    style: TextStyle(
                      color: Color(0xFF64748B),
                      fontSize: 12,
                      height: 1.35,
                    ),
                  ),
                ],
              ),
            ),
            Switch.adaptive(
              value: value,
              activeThumbColor: const Color(0xFF2563EB),
              onChanged: enabled ? onChanged : null,
            ),
          ],
        ),
      ),
    );
  }
}

class _LoginBackgroundPainter extends CustomPainter {
  const _LoginBackgroundPainter();

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    final basePaint = Paint()
      ..shader = const LinearGradient(
        colors: [Color(0xFFF8FAFC), Color(0xFFEFF6F2), Color(0xFFFFFBEB)],
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
      ).createShader(rect);

    canvas.drawRect(rect, basePaint);

    final topPath = Path()
      ..moveTo(0, 0)
      ..lineTo(size.width, 0)
      ..lineTo(size.width, size.height * 0.34)
      ..cubicTo(
        size.width * 0.76,
        size.height * 0.25,
        size.width * 0.55,
        size.height * 0.42,
        size.width * 0.28,
        size.height * 0.31,
      )
      ..cubicTo(
        size.width * 0.12,
        size.height * 0.24,
        size.width * 0.05,
        size.height * 0.29,
        0,
        size.height * 0.25,
      )
      ..close();
    canvas.drawPath(
      topPath,
      Paint()..color = const Color(0xFFDBEAFE).withValues(alpha: 0.72),
    );

    final bottomPath = Path()
      ..moveTo(0, size.height * 0.72)
      ..cubicTo(
        size.width * 0.24,
        size.height * 0.64,
        size.width * 0.48,
        size.height * 0.82,
        size.width,
        size.height * 0.68,
      )
      ..lineTo(size.width, size.height)
      ..lineTo(0, size.height)
      ..close();
    canvas.drawPath(
      bottomPath,
      Paint()..color = const Color(0xFFCCFBF1).withValues(alpha: 0.58),
    );

    final linePaint = Paint()
      ..color = Colors.white.withValues(alpha: 0.45)
      ..strokeWidth = 1;
    const spacing = 34.0;
    for (var x = -size.height; x < size.width; x += spacing) {
      canvas.drawLine(
        Offset(x, 0),
        Offset(x + size.height, size.height),
        linePaint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
