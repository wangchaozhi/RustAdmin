import 'package:flutter/material.dart';

import 'app/mobile_app.dart';
import 'core/api_config.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await ApiConfig.load();
  runApp(const MobileApp());
}
