import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'screens/home_screen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent,
      statusBarIconBrightness: Brightness.light,
      systemNavigationBarColor: Colors.transparent,
      systemNavigationBarIconBrightness: Brightness.light,
    ),
  );
  runApp(const DropBarMobileApp());
}

class DropBarMobileApp extends StatelessWidget {
  const DropBarMobileApp({super.key});

  @override
  Widget build(BuildContext context) {
    // Android Material 3 (Material You) Theme
    const Color androidSeedColor = Color(0xFF0B57D0); // Google Android System Blue

    final darkColorScheme = ColorScheme.fromSeed(
      seedColor: androidSeedColor,
      brightness: Brightness.dark,
    ).copyWith(
      surface: const Color(0xFF111318),
      surfaceContainer: const Color(0xFF1D2024),
      surfaceContainerHigh: const Color(0xFF282A2F),
      surfaceContainerHighest: const Color(0xFF33353A),
      primary: const Color(0xFFA8C7FA),
      onPrimary: const Color(0xFF003062),
      primaryContainer: const Color(0xFF00468B),
      onPrimaryContainer: const Color(0xFFD6E3FF),
      secondaryContainer: const Color(0xFF3B4858),
      onSecondaryContainer: const Color(0xFFD7E3F8),
    );

    final lightColorScheme = ColorScheme.fromSeed(
      seedColor: androidSeedColor,
      brightness: Brightness.light,
    ).copyWith(
      surface: const Color(0xFFFDFBFF),
      surfaceContainer: const Color(0xFFF0F4F9),
      surfaceContainerHigh: const Color(0xFFE8EEF4),
      surfaceContainerHighest: const Color(0xFFDFE4E9),
      primary: const Color(0xFF0B57D0),
      onPrimary: Colors.white,
      primaryContainer: const Color(0xFFD6E3FF),
      onPrimaryContainer: const Color(0xFF001B3E),
    );

    return MaterialApp(
      title: 'DropBar',
      debugShowCheckedModeBanner: false,
      themeMode: ThemeMode.system,
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: lightColorScheme,
        scaffoldBackgroundColor: lightColorScheme.surface,
        appBarTheme: AppBarTheme(
          centerTitle: false,
          elevation: 0,
          backgroundColor: lightColorScheme.surface,
          foregroundColor: lightColorScheme.onSurface,
          titleTextStyle: TextStyle(
            fontSize: 22,
            fontWeight: FontWeight.w600,
            color: lightColorScheme.onSurface,
            letterSpacing: -0.2,
          ),
        ),
        cardTheme: CardTheme(
          elevation: 0,
          color: lightColorScheme.surfaceContainer,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(20),
          ),
        ),
      ),
      darkTheme: ThemeData(
        useMaterial3: true,
        colorScheme: darkColorScheme,
        scaffoldBackgroundColor: darkColorScheme.surface,
        appBarTheme: AppBarTheme(
          centerTitle: false,
          elevation: 0,
          backgroundColor: darkColorScheme.surface,
          foregroundColor: darkColorScheme.onSurface,
          titleTextStyle: TextStyle(
            fontSize: 22,
            fontWeight: FontWeight.w600,
            color: darkColorScheme.onSurface,
            letterSpacing: -0.2,
          ),
        ),
        cardTheme: CardTheme(
          elevation: 0,
          color: darkColorScheme.surfaceContainer,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(20),
          ),
        ),
      ),
      home: const HomeScreen(),
    );
  }
}


