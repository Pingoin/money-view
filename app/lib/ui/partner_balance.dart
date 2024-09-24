import 'package:app/application_state.dart';
import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:app/ui/balance_chart.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class PartnerBalanceWidget extends StatefulWidget {
  const PartnerBalanceWidget({
    super.key,
  });

  @override
  State<PartnerBalanceWidget> createState() => _PartnerBalanceWidgetState();
}

class _PartnerBalanceWidgetState extends State<PartnerBalanceWidget> {
  List<BalanceInformation> partnerExpenses = [];
  List<BalanceInformation> partnerIncome = [];
  double totalExpenses = 0.0;
  double totalIncome = 0.0;
  bool isEmpty = true;

  @override
  Widget build(BuildContext context) {
    var appState = context.watch<ApplicationState>();
    if (isEmpty) {
      appState.moneyViewClient.getPartnerBalance(Empty()).then((response) {
        setState(() {
          partnerExpenses = response.partnerExpenses;
          partnerExpenses
              .sort((a, b) => -a.balance.abs().compareTo(b.balance.abs()));
          partnerIncome = response.partnerIncome;
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
