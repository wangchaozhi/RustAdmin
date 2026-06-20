import 'dart:io';
import 'dart:async';

import 'package:ffmpeg_kit_flutter_new/ffmpeg_kit.dart';
import 'package:ffmpeg_kit_flutter_new/return_code.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import 'video_audio_converter_models.dart';

bool get isVideoAudioConversionSupported => true;

Future<VideoAudioConversionResult> convertVideoToAudio({
  required PlatformFile file,
  required AudioOutputFormat format,
}) async {
  final inputPath = file.path;
  if (inputPath == null || inputPath.isEmpty) {
    throw Exception('当前平台无法读取视频文件路径');
  }

  final outputDirectory =
      await getDownloadsDirectory() ?? await getApplicationDocumentsDirectory();
  final sourceName = p.basenameWithoutExtension(inputPath);
  final timestamp = DateTime.now().millisecondsSinceEpoch;
  final fileName =
      '${_safeFileName(sourceName)}_$timestamp.${format.extension}';
  final outputPath = p.join(outputDirectory.path, fileName);
  final command = _buildCommand(inputPath, outputPath, format);

  final session = await _execute(command);
  final returnCode = await session.getReturnCode();
  if (ReturnCode.isSuccess(returnCode)) {
    return VideoAudioConversionResult(
      outputPath: outputPath,
      fileName: fileName,
    );
  }

  final logs = await session.getOutput();
  final reason = logs.trim().isEmpty ? '转码失败' : logs.trim();
  final outputFile = File(outputPath);
  if (await outputFile.exists()) {
    await outputFile.delete();
  }
  throw Exception(reason);
}

Future<dynamic> _execute(String command) {
  final completer = Completer<dynamic>();
  FFmpegKit.executeAsync(command, (session) {
    if (!completer.isCompleted) {
      completer.complete(session);
    }
  });
  return completer.future;
}

String _buildCommand(
  String inputPath,
  String outputPath,
  AudioOutputFormat format,
) {
  final input = _quote(inputPath);
  final output = _quote(outputPath);
  return switch (format) {
    AudioOutputFormat.m4a => '-y -i $input -vn -c:a aac -b:a 192k $output',
    AudioOutputFormat.mp3 => '-y -i $input -vn -c:a libmp3lame -q:a 2 $output',
    AudioOutputFormat.wav => '-y -i $input -vn -c:a pcm_s16le $output',
  };
}

String _quote(String value) => '"${value.replaceAll('"', r'\"')}"';

String _safeFileName(String value) {
  final cleaned = value
      .replaceAll(RegExp(r'[\\/:*?"<>|]+'), '_')
      .replaceAll(RegExp(r'\s+'), '_')
      .trim();
  return cleaned.isEmpty ? 'audio' : cleaned;
}
