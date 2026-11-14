import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:open_filex/open_filex.dart';
import 'package:path_provider/path_provider.dart';

class ReceivedItem {
  final String filename;
  final String path;
  final int size;
  final String sender;
  final DateTime receivedAt;
  final bool isFile;
  final String? textContent;

  ReceivedItem({
    required this.filename,
    required this.path,
    required this.size,
    required this.sender,
    required this.receivedAt,
    this.isFile = true,
    this.textContent,
  });
}

class MobileReceiverServer {
  final int port;
  HttpServer? _server;
  final StreamController<ReceivedItem> _itemsController =
      StreamController<ReceivedItem>.broadcast();
  final FlutterLocalNotificationsPlugin _notifications =
      FlutterLocalNotificationsPlugin();

  MobileReceiverServer({this.port = 53319});

  Stream<ReceivedItem> get itemStream => _itemsController.stream;

  Future<void> initNotifications() async {
    const androidSettings = AndroidInitializationSettings('@mipmap/ic_launcher');
    const iosSettings = DarwinInitializationSettings();
    const settings = InitializationSettings(android: androidSettings, iOS: iosSettings);

    await _notifications.initialize(
      settings,
      onDidReceiveNotificationResponse: (response) {
        if (response.payload != null && File(response.payload!).existsSync()) {
          OpenFilex.open(response.payload!);
        }
      },
    );

    // Create high-importance notification channel for instant popups on Android
    final androidPlugin = _notifications
        .resolvePlatformSpecificImplementation<AndroidFlutterLocalNotificationsPlugin>();
    await androidPlugin?.createNotificationChannel(
      const AndroidNotificationChannel(
        'dropbar_transfers',
        'DropBar Transfers',
        description: 'Instant popup alerts for files received from Arch Linux',
        importance: Importance.max,
        playSound: true,
        enableVibration: true,
      ),
    );
  }

  Future<Directory> getDropBarFolder() async {
    // 1. On Android, use the public Download/DropBar folder so it is visible in File Manager & Gallery
    if (Platform.isAndroid) {
      final downloadDir = Directory("/storage/emulated/0/Download/DropBar");
      if (!await downloadDir.exists()) {
        try {
          await downloadDir.create(recursive: true);
        } catch (_) {
          // Fallback if scoped storage blocks direct path
        }
      }
      if (await downloadDir.exists()) {
        return downloadDir;
      }
    }

    // 2. Standard fallback
    final baseDir = await getExternalStorageDirectory() ??
        await getApplicationDocumentsDirectory();
    final dropDir = Directory("${baseDir.path}/DropBar");
    if (!await dropDir.exists()) {
      await dropDir.create(recursive: true);
    }
    return dropDir;
  }

  Future<void> showPopupNotification(String title, String body, String filePath) async {
    final androidDetails = AndroidNotificationDetails(
      'dropbar_transfers',
      'DropBar Transfers',
      channelDescription: 'Instant popup alerts for files received from Arch Linux',
      importance: Importance.max,
      priority: Priority.high,
      fullScreenIntent: true, // Heads-up popup bar
      ticker: 'DropBar Transfer',
      icon: '@mipmap/ic_launcher',
    );
    final details = NotificationDetails(
      android: androidDetails,
      iOS: const DarwinNotificationDetails(presentAlert: true, presentBanner: true),
    );

    await _notifications.show(
      DateTime.now().millisecond,
      title,
      body,
      details,
      payload: filePath,
    );
  }

  Future<void> start() async {
    await initNotifications();

    try {
      _server = await HttpServer.bind(InternetAddress.anyIPv4, port);
      print("DropBar Mobile Receiver active on port $port");

      _server?.listen((HttpRequest request) async {
        final path = request.uri.path;
        request.response.headers.add("Access-Control-Allow-Origin", "*");

        if (request.method == 'GET' && path == '/api/v1/ping') {
          request.response
            ..statusCode = HttpStatus.ok
            ..headers.contentType = ContentType.json
            ..write(jsonEncode({"status": "ok", "device": "P4", "device_type": "Android"}))
            ..close();
          return;
        }

        if (request.method == 'POST' && path == '/api/v1/transfer/text') {
          final content = await utf8.decoder.bind(request).join();
          final data = jsonDecode(content) as Map<String, dynamic>;
          final text = data['text'] as String? ?? '';
          final sender = data['sender_name'] as String? ?? 'Arch Linux';

          final item = ReceivedItem(
            filename: 'Text snippet',
            path: '',
            size: utf8.encode(text).length,
            sender: sender,
            receivedAt: DateTime.now(),
            isFile: false,
            textContent: text,
          );
          _itemsController.add(item);

          await showPopupNotification("💬 DropBar: New Message", text, "");

          request.response
            ..statusCode = HttpStatus.ok
            ..headers.contentType = ContentType.json
            ..write(jsonEncode({"success": true}))
            ..close();
          return;
        }

        if (request.method == 'POST' && path == '/api/v1/transfer/file') {
          final targetFolder = await getDropBarFolder();

          // Read bytes from incoming stream
          final bytes = <int>[];
          await for (final chunk in request) {
            bytes.addAll(chunk);
          }

          // Parse multipart filename if present, or generate safe timestamp name
          String filename = "DropBar_File_${DateTime.now().millisecondsSinceEpoch}.bin";
          int fileStart = 0;
          int fileEnd = bytes.length;

          final headerStr = String.fromCharCodes(bytes.take(1024));
          final filenameMatch = RegExp(r'filename="([^"]+)"').firstMatch(headerStr);
          if (filenameMatch != null) {
            filename = filenameMatch.group(1)!;

            // Find end of header (\r\n\r\n or \n\n)
            final headerEndIndex = headerStr.indexOf("\r\n\r\n");
            if (headerEndIndex != -1) {
              fileStart = headerEndIndex + 4;
            } else {
              final altEndIndex = headerStr.indexOf("\n\n");
              if (altEndIndex != -1) fileStart = altEndIndex + 2;
            }

            // Strip trailing boundary if present
            final boundaryMatch = RegExp(r'--[a-zA-Z0-9_-]+--').firstMatch(
                String.fromCharCodes(bytes.skip(bytes.length > 256 ? bytes.length - 256 : 0)));
            if (boundaryMatch != null) {
              fileEnd = bytes.length - boundaryMatch.group(0)!.length - 4;
            }
          }

          if (fileEnd > fileStart) {
            final fileData = bytes.sublist(fileStart, fileEnd);
            final targetFile = File("${targetFolder.path}/$filename");
            await targetFile.writeAsBytes(fileData);

            final sizeKb = (fileData.length / 1024).toStringAsFixed(1);
            final item = ReceivedItem(
              filename: filename,
              path: targetFile.path,
              size: fileData.length,
              sender: "Arch Linux",
              receivedAt: DateTime.now(),
            );
            _itemsController.add(item);

            // Trigger instant heads-up popup alert on P4 screen!
            await showPopupNotification(
              "📥 DropBar: Received $filename",
              "Saved to ${targetFolder.path} ($sizeKb KB). Tap to open.",
              targetFile.path,
            );

            request.response
              ..statusCode = HttpStatus.ok
              ..headers.contentType = ContentType.json
              ..write(jsonEncode({"success": true, "path": targetFile.path}))
              ..close();
            return;
          }
        }

        request.response
          ..statusCode = HttpStatus.notFound
          ..close();
      });
    } catch (e) {
      print("Failed to start mobile server: $e");
    }
  }

  void stop() {
    _server?.close(force: true);
    _itemsController.close();
  }
}
