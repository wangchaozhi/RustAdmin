import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';

import 'video_audio_converter_models.dart';
import 'video_audio_converter_service.dart';

class VideoAudioConverterPage extends StatefulWidget {
  const VideoAudioConverterPage({super.key});

  @override
  State<VideoAudioConverterPage> createState() =>
      _VideoAudioConverterPageState();
}

class _VideoAudioConverterPageState extends State<VideoAudioConverterPage> {
  AudioOutputFormat _format = AudioOutputFormat.m4a;
  String? _inputPath;
  String? _inputName;
  String? _outputPath;
  bool _converting = false;

  Future<void> _pickVideo() async {
    final result = await FilePicker.pickFiles(
      type: FileType.video,
      allowMultiple: false,
      withData: false,
    );
    if (result == null || result.files.isEmpty) return;

    final file = result.files.single;
    if (file.path == null || file.path!.isEmpty) {
      _showMessage('当前平台无法读取视频文件路径');
      return;
    }

    setState(() {
      _inputPath = file.path;
      _inputName = file.name;
      _outputPath = null;
    });
  }

  Future<void> _convert() async {
    final inputPath = _inputPath;
    if (inputPath == null || inputPath.isEmpty) {
      _showMessage('请先选择视频文件');
      return;
    }
    if (!isVideoAudioConversionSupported) {
      _showMessage('当前环境暂不支持本地视频转音频');
      return;
    }

    setState(() => _converting = true);
    try {
      final result = await convertVideoToAudio(
        inputPath: inputPath,
        format: _format,
      );
      if (!mounted) return;
      setState(() => _outputPath = result.outputPath);
      _showMessage('已生成 ${result.fileName}');
    } catch (e) {
      if (!mounted) return;
      _showMessage('转换失败：$e');
    } finally {
      if (mounted) setState(() => _converting = false);
    }
  }

  void _showMessage(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('视频转音频'), centerTitle: false),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 28),
          children: [
            const _IntroPanel(),
            const SizedBox(height: 16),
            _FormatSelector(
              value: _format,
              enabled: !_converting,
              onChanged: (value) => setState(() => _format = value),
            ),
            const SizedBox(height: 16),
            _FilePanel(
              inputName: _inputName,
              outputPath: _outputPath,
              converting: _converting,
              onPick: _pickVideo,
              onConvert: _convert,
            ),
            if (!isVideoAudioConversionSupported) ...[
              const SizedBox(height: 16),
              const _UnsupportedPanel(),
            ],
          ],
        ),
      ),
    );
  }
}

class _IntroPanel extends StatelessWidget {
  const _IntroPanel();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        color: const Color(0xFF111827),
        borderRadius: BorderRadius.circular(20),
      ),
      child: const Row(
        children: [
          Icon(Icons.graphic_eq_rounded, color: Colors.white, size: 34),
          SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '提取视频中的音轨',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.w800,
                    fontSize: 18,
                  ),
                ),
                SizedBox(height: 4),
                Text(
                  '选择手机里的视频文件，导出为 M4A、MP3 或 WAV 音频。',
                  style: TextStyle(color: Color(0xFFD1D5DB), height: 1.45),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _FormatSelector extends StatelessWidget {
  const _FormatSelector({
    required this.value,
    required this.enabled,
    required this.onChanged,
  });

  final AudioOutputFormat value;
  final bool enabled;
  final ValueChanged<AudioOutputFormat> onChanged;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: const Color(0xFFE5E7EB)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            '输出格式',
            style: TextStyle(
              color: Color(0xFF111827),
              fontSize: 16,
              fontWeight: FontWeight.w800,
            ),
          ),
          const SizedBox(height: 12),
          SegmentedButton<AudioOutputFormat>(
            segments: AudioOutputFormat.values
                .map(
                  (format) =>
                      ButtonSegment(value: format, label: Text(format.label)),
                )
                .toList(),
            selected: {value},
            onSelectionChanged: enabled
                ? (selected) => onChanged(selected.first)
                : null,
          ),
        ],
      ),
    );
  }
}

class _FilePanel extends StatelessWidget {
  const _FilePanel({
    required this.inputName,
    required this.outputPath,
    required this.converting,
    required this.onPick,
    required this.onConvert,
  });

  final String? inputName;
  final String? outputPath;
  final bool converting;
  final VoidCallback onPick;
  final VoidCallback onConvert;

  @override
  Widget build(BuildContext context) {
    final hasInput = inputName != null && inputName!.isNotEmpty;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: const Color(0xFFE5E7EB)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          OutlinedButton.icon(
            onPressed: converting ? null : onPick,
            icon: const Icon(Icons.video_file_rounded),
            label: Text(hasInput ? inputName! : '选择视频文件'),
            style: OutlinedButton.styleFrom(
              minimumSize: const Size.fromHeight(52),
              alignment: Alignment.centerLeft,
            ),
          ),
          const SizedBox(height: 12),
          FilledButton.icon(
            onPressed: converting ? null : onConvert,
            icon: converting
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.music_note_rounded),
            label: Text(converting ? '正在转换...' : '开始转换'),
            style: FilledButton.styleFrom(
              minimumSize: const Size.fromHeight(52),
              backgroundColor: const Color(0xFF2563EB),
            ),
          ),
          if (outputPath != null && outputPath!.isNotEmpty) ...[
            const SizedBox(height: 14),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: const Color(0xFFEFF6FF),
                borderRadius: BorderRadius.circular(14),
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Icon(
                    Icons.check_circle_rounded,
                    color: Color(0xFF2563EB),
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      outputPath!,
                      style: const TextStyle(
                        color: Color(0xFF1E3A8A),
                        height: 1.4,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _UnsupportedPanel extends StatelessWidget {
  const _UnsupportedPanel();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: const Color(0xFFFFFBEB),
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: const Color(0xFFFDE68A)),
      ),
      child: const Row(
        children: [
          Icon(Icons.info_rounded, color: Color(0xFFB45309)),
          SizedBox(width: 10),
          Expanded(
            child: Text(
              '浏览器环境不能直接调用本地 FFmpeg 转码。请在 Android 或 iOS App 中使用此功能。',
              style: TextStyle(color: Color(0xFF92400E), height: 1.45),
            ),
          ),
        ],
      ),
    );
  }
}
