import 'package:app/application_state.dart';
import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:app/ui/balance_chart.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class TagBalanceWidget extends StatefulWidget {
  const TagBalanceWidget({
    super.key,
  });

  @override
  State<TagBalanceWidget> createState() => _TagBalanceWidgetState();
}

class _TagBalanceWidgetState extends State<TagBalanceWidget> {
  List<BalanceInformation> partnerExpenses = [];
  List<BalanceInformation> partnerIncome = [];
  double totalExpenses = 0.0;
  double totalIncome = 0.0;
  bool isEmpty = true;

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<ApplicationState>();
    if (isEmpty) {
      appState.moneyViewClient.getTagBalance(Empty()).then((response) {
        setState(() {
          partnerExpenses = response.expenses;
          partnerExpenses
              .sort((a, b) => -a.balance.abs().compareTo(b.balance.abs()));
          partnerIncome = response.income;
          partnerIncome
              .sort((a, b) => -a.balance.abs().compareTo(b.balance.abs()));
          totalExpenses = response.totalExpenses;
          totalIncome = response.totalIncome;
          isEmpty = false;
        });
      });
    }

    return Center(
        child: ListView(
      children: [
        BalanceChart(
          data: partnerExpenses,
          threshold: 200,
        ),
        BalanceChart(
          data: partnerIncome,
          threshold: 200,
        ),
      ],
    ));
  }
}
