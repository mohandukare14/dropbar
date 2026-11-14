import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:open_filex/open_filex.dart';
import 'package:share_plus/share_plus.dart';
import '../models/device_info.dart';
import '../services/discovery_service.dart';
import '../services/receiver_service.dart';
import '../services/transfer_service.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final DiscoveryService _discoveryService = DiscoveryService();
  final MobileReceiverServer _receiverServer = MobileReceiverServer();
  late final TransferService _transferService;

  final List<File> _stashedFiles = [];
  final List<ReceivedItem> _receivedItems = [];
  DeviceInfo? _selectedPeer;
  bool _isTransferring = false;
  final TextEditingController _textController = TextEditingController();

  final DeviceInfo _localDevice = DeviceInfo(
    id: "mobile_${DateTime.now().millisecondsSinceEpoch}",
    name: Platform.isAndroid ? "Android Device" : "iOS Device",
    deviceType: Platform.isAndroid ? "Android" : "IOS",
    ip: "127.0.0.1",
    port: 53319,
    lastSeen: DateTime.now(),
  );

  @override
  void initState() {
    super.initState();
    _transferService = TransferService(localDeviceName: _localDevice.name);
    _initServices();
  }

  Future<void> _initServices() async {
    await _discoveryService.start(_localDevice);
    await _receiverServer.start();

    _receiverServer.itemStream.listen((item) {
      setState(() {
        _receivedItems.insert(0, item);
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text("📥 Received: ${item.filename} from ${item.sender}"),
          backgroundColor: const Color(0xFF00FFCC),
        ),
      );
    });
  }

  @override
  void dispose() {
    _discoveryService.stop();
    _receiverServer.stop();
    _textController.dispose();
    super.dispose();
  }

  Future<void> _pickFiles() async {
    final result = await FilePicker.platform.pickFiles(allowMultiple: true);
    if (result != null && result.paths.isNotEmpty) {
      setState(() {
        for (final path in result.paths) {
          if (path != null) {
            _stashedFiles.add(File(path));
          }
        }
      });
    }
  }

  Future<void> _sendStashedFiles() async {
    if (_selectedPeer == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text("⚠️ Select an Arch Linux peer first!")),
      );
      return;
    }

    if (_stashedFiles.isEmpty) return;

    setState(() => _isTransferring = true);
    int successCount = 0;

    for (final file in List<File>.from(_stashedFiles)) {
      final success = await _transferService.sendFile(
        target: _selectedPeer!,
        file: file,
      );
      if (success) {
        successCount++;
        setState(() {
          _stashedFiles.remove(file);
        });
      }
    }

    setState(() => _isTransferring = false);

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text("🚀 Sent $successCount file(s) to ${_selectedPeer!.name}!"),
          backgroundColor: Colors.greenAccent.shade700,
        ),
      );
    }
  }

  Future<void> _sendText() async {
    if (_selectedPeer == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text("⚠️ Select an Arch Linux peer first!")),
      );
      return;
    }

    final text = _textController.text.trim();
    if (text.isEmpty) return;

    final isUrl = text.startsWith("http://") || text.startsWith("https://");
    final success = await _transferService.sendText(
      target: _selectedPeer!,
      text: text,
      isUrl: isUrl,
    );

    if (success && mounted) {
      _textController.clear();
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text("✅ Text beamed to Arch Linux!"),
          backgroundColor: Colors.tealAccent,
        ),
      );
    }
  }

  Future<void> _pasteClipboard() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    if (data?.text != null && data!.text!.isNotEmpty) {
      _textController.text = data.text!;
      _sendText();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0F141C),
      appBar: AppBar(
        backgroundColor: const Color(0xFF161E2E),
        elevation: 0,
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(6),
              decoration: BoxDecoration(
                color: const Color(0xFF00FFCC).withOpacity(0.15),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Icon(Icons.swap_vert_circle, color: Color(0xFF00FFCC)),
            ),
            const SizedBox(width: 12),
            const Text(
              "DropBar",
              style: TextStyle(fontWeight: FontWeight.bold, letterSpacing: 0.8),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.add_link),
            tooltip: "Manual IP Connect",
            onPressed: _showManualConnectDialog,
          ),
        ],
      ),
      body: StreamBuilder<List<DeviceInfo>>(
        stream: _discoveryService.peersStream,
        initialData: _discoveryService.currentPeers,
        builder: (context, snapshot) {
          final peers = snapshot.data ?? [];
          if (_selectedPeer == null && peers.isNotEmpty) {
            _selectedPeer = peers.first;
          }

          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Peer Selector Bar
              _buildPeerSelector(peers),

              // Main Drop & Stash Area
              Expanded(
                child: ListView(
                  padding: const EdgeInsets.all(16),
                  children: [
                    _buildDropZone(),
                    const SizedBox(height: 16),
                    _buildQuickTextBar(),
                    if (_stashedFiles.isNotEmpty) ...[
                      const SizedBox(height: 20),
                      _buildStashList(),
                    ],
                    if (_receivedItems.isNotEmpty) ...[
                      const SizedBox(height: 24),
                      _buildReceivedHistory(),
                    ],
                  ],
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildPeerSelector(List<DeviceInfo> peers) {
    return Container(
      color: const Color(0xFF161E2E),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
      child: Row(
        children: [
          Container(
            width: 10,
            height: 10,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: peers.isNotEmpty ? const Color(0xFF00FFCC) : Colors.orangeAccent,
              boxShadow: [
                BoxShadow(
                  color: (peers.isNotEmpty ? const Color(0xFF00FFCC) : Colors.orangeAccent)
                      .withOpacity(0.5),
                  blurRadius: 6,
                )
              ],
            ),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: peers.isEmpty
                ? const Text(
                    "Searching for Arch Linux DropBar...",
                    style: TextStyle(color: Colors.white70, fontSize: 13),
                  )
                : DropdownButtonHideUnderline(
                    child: DropdownButton<DeviceInfo>(
                      value: _selectedPeer,
                      dropdownColor: const Color(0xFF1E293B),
                      isDense: true,
                      style: const TextStyle(color: Colors.white, fontSize: 13),
                      items: peers.map((p) {
                        return DropdownMenuItem(
                          value: p,
                          child: Text("🖥️ ${p.name} (${p.ip})"),
                        );
                      }).toList(),
                      onChanged: (val) {
                        setState(() => _selectedPeer = val);
                      },
                    ),
                  ),
          ),
        ],
      ),
    );
  }

  Widget _buildDropZone() {
    return DragTarget<dynamic>(
      onAcceptWithDetails: (details) {
        // Accepts drops from other apps in split screen or file manager
      },
      builder: (context, candidateData, rejectedData) {
        return GestureDetector(
          onTap: _pickFiles,
          child: Container(
            height: 160,
            decoration: BoxDecoration(
              color: const Color(0xFF161E2E),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: candidateData.isNotEmpty
                    ? const Color(0xFF00FFCC)
                    : const Color(0xFF2E3A4E),
                width: 2,
                style: BorderStyle.solid,
              ),
            ),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(
                  Icons.file_upload_outlined,
                  size: 48,
                  color: const Color(0xFF00FFCC).withOpacity(0.9),
                ),
                const SizedBox(height: 12),
                const Text(
                  "Drop files here or Tap to browse",
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.w600,
                    fontSize: 15,
                  ),
                ),
                const SizedBox(height: 4),
                const Text(
                  "Photos, PDFs, videos, or archives to send to Arch Linux",
                  style: TextStyle(color: Colors.white38, fontSize: 11),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildQuickTextBar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xFF161E2E),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xFF2E3A4E)),
      ),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: _textController,
              style: const TextStyle(color: Colors.white, fontSize: 13),
              decoration: const InputDecoration(
                hintText: "Beam text snippet or URL...",
                hintStyle: TextStyle(color: Colors.white38),
                border: InputBorder.none,
                isDense: true,
              ),
              onSubmitted: (_) => _sendText(),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.paste, color: Colors.white70, size: 20),
            tooltip: "Paste & Beam",
            onPressed: _pasteClipboard,
          ),
          IconButton(
            icon: const Icon(Icons.send_rounded, color: Color(0xFF00FFCC), size: 20),
            tooltip: "Send",
            onPressed: _sendText,
          ),
        ],
      ),
    );
  }

  Widget _buildStashList() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              "STASHED FILES (${_stashedFiles.length})",
              style: const TextStyle(
                color: Colors.white70,
                fontSize: 12,
                fontWeight: FontWeight.bold,
                letterSpacing: 1.1,
              ),
            ),
            TextButton(
              onPressed: () => setState(() => _stashedFiles.clear()),
              child: const Text("Clear", style: TextStyle(color: Colors.redAccent, fontSize: 12)),
            )
          ],
        ),
        ..._stashedFiles.map((file) {
          final name = file.uri.pathSegments.last;
          return Container(
            margin: const EdgeInsets.only(bottom: 8),
            decoration: BoxDecoration(
              color: const Color(0xFF1A2333),
              borderRadius: BorderRadius.circular(8),
            ),
            child: ListTile(
              dense: true,
              leading: const Icon(Icons.insert_drive_file, color: Color(0xFF00FFCC)),
              title: Text(name, style: const TextStyle(color: Colors.white, fontSize: 13)),
              trailing: IconButton(
                icon: const Icon(Icons.close, size: 18, color: Colors.white38),
                onPressed: () => setState(() => _stashedFiles.remove(file)),
              ),
            ),
          );
        }),
        const SizedBox(height: 10),
        SizedBox(
          width: double.infinity,
          height: 46,
          child: ElevatedButton.icon(
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF00FFCC),
              foregroundColor: Colors.black,
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
            ),
            icon: _isTransferring
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black),
                  )
                : const Icon(Icons.cloud_upload),
            label: Text(
              _isTransferring ? "Beam in progress..." : "Beam to Arch Linux",
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
            onPressed: _isTransferring ? null : _sendStashedFiles,
          ),
        ),
      ],
    );
  }

  Widget _buildReceivedHistory() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          "RECEIVED FROM ARCH LINUX",
          style: TextStyle(
            color: Colors.white70,
            fontSize: 12,
            fontWeight: FontWeight.bold,
            letterSpacing: 1.1,
          ),
        ),
        const SizedBox(height: 8),
        ..._receivedItems.map((item) {
          return Container(
            margin: const EdgeInsets.only(bottom: 8),
            decoration: BoxDecoration(
              color: const Color(0xFF1E293B),
              borderRadius: BorderRadius.circular(10),
            ),
            child: ListTile(
              leading: Icon(
                item.isFile ? Icons.download_done : Icons.text_snippet,
                color: Colors.greenAccent,
              ),
              title: Text(
                item.filename,
                style: const TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.w600),
              ),
              subtitle: Text(
                item.isFile
                    ? "${(item.size / 1024).toStringAsFixed(1)} KB • ${item.sender}"
                    : "${item.textContent} • ${item.sender}",
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(color: Colors.white54, fontSize: 11),
              ),
              trailing: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (item.isFile)
                    IconButton(
                      icon: const Icon(Icons.open_in_new, color: Colors.white70, size: 18),
                      onPressed: () => OpenFilex.open(item.path),
                    ),
                  IconButton(
                    icon: const Icon(Icons.share, color: Colors.white70, size: 18),
                    onPressed: () {
                      if (item.isFile) {
                        Share.shareXFiles([XFile(item.path)]);
                      } else if (item.textContent != null) {
                        Share.share(item.textContent!);
                      }
                    },
                  ),
                ],
              ),
            ),
          );
        }),
      ],
    );
  }

  void _showManualConnectDialog() {
    final ipCtrl = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF1E293B),
        title: const Text("Connect to Arch IP", style: TextStyle(color: Colors.white)),
        content: TextField(
          controller: ipCtrl,
          style: const TextStyle(color: Colors.white),
          decoration: const InputDecoration(
            hintText: "e.g. 192.168.1.100",
            hintStyle: TextStyle(color: Colors.white38),
          ),
        ),
        actions: [
          TextButton(
            child: const Text("Cancel"),
            onPressed: () => Navigator.pop(ctx),
          ),
          ElevatedButton(
            child: const Text("Connect"),
            onPressed: () {
              if (ipCtrl.text.isNotEmpty) {
                _discoveryService.addManualPeer(ipCtrl.text.trim(), 53318, "Arch-Manual");
              }
              Navigator.pop(ctx);
            },
          )
        ],
      ),
    );
  }
}

