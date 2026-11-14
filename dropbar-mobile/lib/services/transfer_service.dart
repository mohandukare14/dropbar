import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import '../models/device_info.dart';

class TransferService {
  final String localDeviceName;

  TransferService({required this.localDeviceName});

  Future<bool> sendText({
    required DeviceInfo target,
    required String text,
    required bool isUrl,
  }) async {
    final uri = Uri.parse("http://${target.ip}:${target.port}/api/v1/transfer/text");
    try {
      final response = await http.post(
        uri,
        headers: {"Content-Type": "application/json"},
        body: jsonEncode({
          "text": text,
          "is_url": isUrl,
          "sender_name": localDeviceName,
        }),
      );
      return response.statusCode == 200;
    } catch (e) {
      print("Send text error: $e");
      return false;
    }
  }

  Future<bool> sendFile({
    required DeviceInfo target,
    required File file,
    Function(double progress)? onProgress,
  }) async {
    final uri = Uri.parse("http://${target.ip}:${target.port}/api/v1/transfer/file");
    try {
      final request = http.MultipartRequest("POST", uri);
      request.fields["sender"] = localDeviceName;

      final length = await file.length();
      final stream = http.ByteStream(file.openRead());

      final multipartFile = http.MultipartFile(
        'file',
        stream,
        length,
        filename: file.uri.pathSegments.last,
      );

      request.files.add(multipartFile);

      final streamedResponse = await request.send();
      final response = await http.Response.fromStream(streamedResponse);
      return response.statusCode == 200;
    } catch (e) {
      print("Send file error: $e");
      return false;
    }
  }
}

