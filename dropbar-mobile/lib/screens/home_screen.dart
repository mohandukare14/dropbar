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
          behavior: SnackBarBehavior.floating,
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
          content: Row(
            children: [
              const Icon(Icons.download_done, color: Colors.white, size: 20),
              const SizedBox(width: 10),
              Expanded(
                child: Text(
                  "Received: ${item.filename}",
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
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
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Icon(Icons.near_me, color: colorScheme.onPrimaryContainer, size: 20),
            ),
            const SizedBox(width: 12),
            const Text("DropBar"),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.wifi_tethering),
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
              _buildPeerSelector(peers, colorScheme),

              // Main Drop & Stash Area
              Expanded(
                child: ListView(
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                  children: [
                    _buildDropZone(colorScheme),
                    const SizedBox(height: 16),
                    _buildQuickTextBar(colorScheme),
                    if (_stashedFiles.isNotEmpty) ...[
                      const SizedBox(height: 20),
                      _buildStashList(colorScheme),
                    ],
                    if (_receivedItems.isNotEmpty) ...[
                      const SizedBox(height: 24),
                      _buildReceivedHistory(colorScheme),
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

  Widget _buildPeerSelector(List<DeviceInfo> peers, ColorScheme colorScheme) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainer,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        children: [
          Container(
            width: 10,
            height: 10,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: peers.isNotEmpty ? Colors.green : Colors.amber,
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: peers.isEmpty
                ? Text(
                    "Searching for Arch Linux DropBar...",
                    style: TextStyle(
                      color: colorScheme.onSurfaceVariant,
                      fontSize: 13,
                    ),
                  )
                : DropdownButtonHideUnderline(
                    child: DropdownButton<DeviceInfo>(
                      value: _selectedPeer,
                      dropdownColor: colorScheme.surfaceContainerHigh,
                      isDense: true,
                      style: TextStyle(color: colorScheme.onSurface, fontSize: 13),
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

  Widget _buildDropZone(ColorScheme colorScheme) {
    return DragTarget<Object>(
      onAcceptWithDetails: (details) {
        // Accepts drops from other apps in split screen or file manager
      },
      builder: (context, candidateData, rejectedData) {
        final isHovered = candidateData.isNotEmpty;
        return Card(
          elevation: 0,
          color: isHovered
              ? colorScheme.primaryContainer.withOpacity(0.5)
              : colorScheme.surfaceContainer,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(24),
            side: BorderSide(
              color: isHovered ? colorScheme.primary : colorScheme.outlineVariant,
              width: isHovered ? 2 : 1,
            ),
          ),
          child: InkWell(
            borderRadius: BorderRadius.circular(24),
            onTap: _pickFiles,
            child: Container(
              height: 160,
              padding: const EdgeInsets.all(20),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: colorScheme.primaryContainer,
                      shape: BoxShape.circle,
                    ),
                    child: Icon(
                      Icons.upload_file_rounded,
                      size: 32,
                      color: colorScheme.onPrimaryContainer,
                    ),
                  ),
                  const SizedBox(height: 12),
                  Text(
                    "Tap to browse or Drop files",
                    style: TextStyle(
                      color: colorScheme.onSurface,
                      fontWeight: FontWeight.w600,
                      fontSize: 15,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    "Beam photos, documents & videos to Arch Linux",
                    style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 12),
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildQuickTextBar(ColorScheme colorScheme) {
    return Card(
      elevation: 0,
      color: colorScheme.surfaceContainer,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(24)),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
        child: Row(
          children: [
            Expanded(
              child: TextField(
                controller: _textController,
                style: TextStyle(color: colorScheme.onSurface, fontSize: 14),
                decoration: InputDecoration(
                  hintText: "Beam text snippet or URL...",
                  hintStyle: TextStyle(color: colorScheme.onSurfaceVariant),
                  border: InputBorder.none,
                  isDense: true,
                ),
                onSubmitted: (_) => _sendText(),
              ),
            ),
            IconButton(
              icon: Icon(Icons.paste_rounded, color: colorScheme.onSurfaceVariant, size: 20),
              tooltip: "Paste & Beam",
              onPressed: _pasteClipboard,
            ),
            IconButton(
              icon: Icon(Icons.arrow_upward_rounded, color: colorScheme.primary, size: 22),
              tooltip: "Send",
              onPressed: _sendText,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStashList(ColorScheme colorScheme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              "QUEUE (${_stashedFiles.length})",
              style: TextStyle(
                color: colorScheme.onSurfaceVariant,
                fontSize: 12,
                fontWeight: FontWeight.w700,
                letterSpacing: 0.8,
              ),
            ),
            TextButton(
              onPressed: () => setState(() => _stashedFiles.clear()),
              child: const Text("Clear all"),
            )
          ],
        ),
        ..._stashedFiles.map((file) {
          final name = file.uri.pathSegments.last;
          return Card(
            elevation: 0,
            margin: const EdgeInsets.only(bottom: 8),
            color: colorScheme.surfaceContainerHigh,
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
            child: ListTile(
              dense: true,
              leading: Icon(Icons.insert_drive_file_outlined, color: colorScheme.primary),
              title: Text(
                name,
                style: TextStyle(color: colorScheme.onSurface, fontSize: 13, fontWeight: FontWeight.w500),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              trailing: IconButton(
                icon: Icon(Icons.close_rounded, size: 18, color: colorScheme.onSurfaceVariant),
                onPressed: () => setState(() => _stashedFiles.remove(file)),
              ),
            ),
          );
        }),
        const SizedBox(height: 8),
        SizedBox(
          width: double.infinity,
          height: 48,
          child: FilledButton.icon(
            icon: _isTransferring
                ? SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: colorScheme.onPrimary,
                    ),
                  )
                : const Icon(Icons.send_rounded),
            label: Text(
              _isTransferring ? "Beaming in progress..." : "Beam to Arch Linux",
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
            onPressed: _isTransferring ? null : _sendStashedFiles,
          ),
        ),
      ],
    );
  }

  Widget _buildReceivedHistory(ColorScheme colorScheme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          "RECEIVED FROM ARCH LINUX",
          style: TextStyle(
            color: colorScheme.onSurfaceVariant,
            fontSize: 12,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.8,
          ),
        ),
        const SizedBox(height: 10),
        ..._receivedItems.map((item) {
          return Card(
            elevation: 0,
            margin: const EdgeInsets.only(bottom: 8),
            color: colorScheme.surfaceContainerHigh,
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
            child: ListTile(
              dense: true,
              leading: Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: colorScheme.secondaryContainer,
                  shape: BoxShape.circle,
                ),
                child: Icon(
                  item.isFile ? Icons.file_present_rounded : Icons.notes_rounded,
                  color: colorScheme.onSecondaryContainer,
                  size: 20,
                ),
              ),
              title: Text(
                item.filename,
                style: TextStyle(
                  color: colorScheme.onSurface,
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              subtitle: Text(
                item.isFile
                    ? "${(item.size / 1024).toStringAsFixed(1)} KB • ${item.sender}"
                    : "${item.textContent} • ${item.sender}",
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(color: colorScheme.onSurfaceVariant, fontSize: 11),
              ),
              trailing: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (item.isFile)
                    IconButton(
                      icon: Icon(Icons.open_in_new_rounded, size: 20, color: colorScheme.primary),
                      tooltip: "Open",
                      onPressed: () => OpenFilex.open(item.path),
                    ),
                  IconButton(
                    icon: Icon(Icons.share_outlined, size: 20, color: colorScheme.onSurfaceVariant),
                    tooltip: "Share",
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
    final colorScheme = Theme.of(context).colorScheme;

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: colorScheme.surfaceContainerHigh,
        title: Text("Connect to Arch IP", style: TextStyle(color: colorScheme.onSurface)),
        content: TextField(
          controller: ipCtrl,
          style: TextStyle(color: colorScheme.onSurface),
          decoration: InputDecoration(
            hintText: "e.g. 10.13.230.209",
            hintStyle: TextStyle(color: colorScheme.onSurfaceVariant.withOpacity(0.5)),
          ),
        ),
        actions: [
          TextButton(
            child: const Text("Cancel"),
            onPressed: () => Navigator.pop(ctx),
          ),
          FilledButton(
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

