// ignore_for_file: avoid_web_libraries_in_flutter, deprecated_member_use

import 'dart:html' as html;

import 'package:file_picker/file_picker.dart';
import 'package:http/http.dart' as http;

import '../../core/api_config.dart';
import 'video_audio_converter_models.dart';

bool get isVideoAudioConversionSupported => true;

Future<VideoAudioConversionResult> convertVideoToAudio({
  required PlatformFile file,
  required AudioOutputFormat format,
}) async {
  final bytes = file.bytes;
  if (bytes == null || bytes.isEmpty) {
    throw Exception('浏览器没有读取到视频文件内容');
  }

  final uri = Uri.parse(
    '${ApiConfig.baseUrl}/api/mobile/video-to-audio?format=${format.extension}',
  );
  final request = http.MultipartRequest('POST', uri)
    ..files.add(
      http.MultipartFile.fromBytes('file', bytes, filename: file.name),
    );
  final streamed = await request.send();
  final response = await http.Response.fromStream(streamed);

  if (response.statusCode < 200 || response.statusCode >= 300) {
    throw Exception(response.body.isEmpty ? '后端转换失败' : response.body);
  }

  final fileName = _outputFileName(file.name, format.extension);
  final blob = html.Blob([response.bodyBytes], format.contentType);
  final url = html.Url.createObjectUrlFromBlob(blob);
  html.AnchorElement(href: url)
    ..download = fileName
    ..click();
  html.Url.revokeObjectUrl(url);

  return VideoAudioConversionResult(
    outputPath: '浏览器已开始下载：$fileName',
    fileName: fileName,
  );
}

String _outputFileName(String inputName, String extension) {
  final dot = inputName.lastIndexOf('.');
  final base = dot > 0 ? inputName.substring(0, dot) : inputName;
  final safe = base
      .replaceAll(RegExp(r'[\\/:*?"<>|]+'), '_')
      .replaceAll(RegExp(r'\s+'), '_')
      .trim();
  return '${safe.isEmpty ? 'audio' : safe}.$extension';
}
