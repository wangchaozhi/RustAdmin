import 'video_audio_converter_models.dart';

bool get isVideoAudioConversionSupported => false;

Future<VideoAudioConversionResult> convertVideoToAudio({
  required String inputPath,
  required AudioOutputFormat format,
}) {
  throw UnsupportedError('当前 Web 环境暂不支持本地视频转音频，请在 Android 或 iOS App 中使用。');
}
